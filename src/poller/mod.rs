use std::fs;
use std::time::Duration;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signature;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSignaturesForAddressConfig;
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use std::str::FromStr;

use crate::cache::SignatureCache;
use crate::common::config::AppState;
use crate::parser::{fetch_transaction, parse_trade};
use crate::strategy::on_trade;

#[derive(Debug)]
pub struct PollerConfig {
    pub poll_interval: Duration,
    pub max_signatures: usize,
}

#[derive(Debug, Clone)]
pub struct PollingServiceConfig {
    pub poll_interval: Duration,
    pub max_signatures: usize,
    pub cache_size: usize,
    pub max_concurrent: usize,
    pub state_path: Option<String>,
}

pub async fn poll_signatures(
    rpc_client: &RpcClient,
    leader_pubkey: &Pubkey,
    config: &PollerConfig,
    before_signature: Option<String>,
) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>, solana_client::client_error::ClientError>
{
    let request_config = RpcSignaturesForAddressConfig {
        before: before_signature,
        limit: Some(config.max_signatures),
        ..RpcSignaturesForAddressConfig::default()
    };

    rpc_client
        .get_signatures_for_address_with_config(leader_pubkey, request_config)
        .await
}

pub async fn start_rpc_polling_service(
    app_state: AppState,
    leader_pubkey: Pubkey,
    config: PollingServiceConfig,
) -> Result<()> {
    let mut cache = SignatureCache::new(config.cache_size);
    let state_path = config
        .state_path
        .as_ref()
        .map(|base| format!("{}.{}", base, leader_pubkey));
    let mut last_seen = load_last_seen(state_path.as_deref());
    let poll_config = PollerConfig {
        poll_interval: config.poll_interval,
        max_signatures: config.max_signatures,
    };

    loop {
        let signatures = fetch_new_signatures(
            &app_state.rpc_nonblocking_client,
            &leader_pubkey,
            &poll_config,
            last_seen.as_ref(),
        )
        .await?;
        let mut ordered_signatures: Vec<RpcConfirmedTransactionStatusWithSignature> =
            signatures.into_iter().rev().collect();

        if let Some(newest) = ordered_signatures.last() {
            last_seen = Some(newest.signature.clone());
            persist_last_seen(state_path.as_deref(), &newest.signature);
        }

        ordered_signatures.retain(|status| !cache.is_seen(&status.signature));

        for status in &ordered_signatures {
            cache.mark_seen(status.signature.clone());
        }

        stream::iter(ordered_signatures)
            .for_each_concurrent(config.max_concurrent, |status| async {
                let signature = match Signature::from_str(&status.signature) {
                    Ok(signature) => signature,
                    Err(err) => {
                        eprintln!("Failed to parse signature {}: {}", status.signature, err);
                        return;
                    }
                };

                match fetch_transaction(&app_state.rpc_nonblocking_client, &signature).await {
                    Ok(transaction) => {
                        if let Some(event) = parse_trade(signature, status.slot, &transaction) {
                            on_trade(&event);
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to fetch transaction {}: {}", status.signature, err);
                    }
                }
            })
            .await;

        tokio::time::sleep(config.poll_interval).await;
    }
}

async fn fetch_new_signatures(
    rpc_client: &RpcClient,
    leader_pubkey: &Pubkey,
    config: &PollerConfig,
    last_seen: Option<&String>,
) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>, solana_client::client_error::ClientError>
{
    let mut before_signature: Option<String> = None;
    let mut collected = Vec::new();

    loop {
        let request_config = RpcSignaturesForAddressConfig {
            before: before_signature.clone(),
            until: last_seen.cloned(),
            limit: Some(config.max_signatures),
            ..RpcSignaturesForAddressConfig::default()
        };
        let mut page = rpc_client
            .get_signatures_for_address_with_config(leader_pubkey, request_config)
            .await?;
        if page.is_empty() {
            break;
        }
        let reached_limit = page.len() == config.max_signatures;
        collected.append(&mut page);
        if !reached_limit {
            break;
        }
        before_signature = collected.last().map(|status| status.signature.clone());
    }

    Ok(collected)
}

fn load_last_seen(path: Option<&str>) -> Option<String> {
    let path = path?;
    match fs::read_to_string(path) {
        Ok(contents) => {
            let trimmed = contents.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Err(_) => None,
    }
}

fn persist_last_seen(path: Option<&str>, signature: &str) {
    let Some(path) = path else { return };
    if let Err(err) = fs::write(path, format!("{}\n", signature)) {
        eprintln!("Failed to persist last seen signature {}: {}", signature, err);
    }
}
