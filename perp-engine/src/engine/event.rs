use rust_decimal::Decimal;

#[derive(Clone, Debug)]
pub enum EngineEvent {
    PriceUpdate(Decimal),
}