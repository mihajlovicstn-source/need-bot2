use anchor_client::solana_sdk::signature::Signature;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding};

use crate::strategy::TradeEvent;

pub async fn fetch_transaction(
    rpc_client: &RpcClient,
    signature: &Signature,
) -> Result<solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta, solana_client::client_error::ClientError>
{
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: None,
        max_supported_transaction_version: None,
    };

    rpc_client
        .get_transaction_with_config(signature, config)
        .await
}

pub fn parse_trade(
    signature: Signature,
    slot: u64,
    transaction: &solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta,
) -> Option<TradeEvent> {
    let meta = transaction.transaction.meta.as_ref()?;
    let pre_balances = match &meta.pre_token_balances {
        OptionSerializer::Some(balances) => balances,
        _ => return None,
    };
    let post_balances = match &meta.post_token_balances {
        OptionSerializer::Some(balances) => balances,
        _ => return None,
    };

    let signature_string = signature.to_string();

    for post_balance in post_balances {
        let mint = post_balance.mint.clone();
        let owner = match &post_balance.owner {
            OptionSerializer::Some(owner) => owner.clone(),
            _ => continue,
        };
        let pre_balance = pre_balances
            .iter()
            .find(|balance| balance.owner == Some(owner.clone()) && balance.mint == mint);

        let pre_amount = pre_balance
            .and_then(|balance| balance.ui_token_amount.ui_amount)
            .unwrap_or(0.0);
        let post_amount = post_balance.ui_token_amount.ui_amount.unwrap_or(0.0);

        let delta = post_amount - pre_amount;
        if delta.abs() > f64::EPSILON {
            let side = if delta > 0.0 { "buy" } else { "sell" };
            return Some(TradeEvent {
                signature: signature_string,
                mint,
                side: side.to_string(),
                amount: delta.abs(),
                slot,
            });
        }
    }

    None
}
