use colored::Colorize;

#[derive(Debug, Clone)]
pub struct TradeEvent {
    pub signature: String,
    pub mint: String,
    pub side: String,
    pub amount: f64,
    pub slot: u64,
}

pub fn on_trade(event: &TradeEvent) {
    let message = format!(
        "Trade detected: {} {} {} (sig: {}, slot: {})",
        event.side, event.amount, event.mint, event.signature, event.slot
    );
    println!("{}", message.green());
}
