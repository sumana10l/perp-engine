use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize, Serializer};

use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PositionType {
    Long,
    Short,
}

fn serialize_decimal<S>(d: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(d.to_f64().unwrap_or(0.0))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub asset: String,

    #[serde(serialize_with = "serialize_decimal")]
    pub entry_price: Decimal,

    #[serde(serialize_with = "serialize_decimal")]
    pub quantity: Decimal,

    #[serde(serialize_with = "serialize_decimal")]
    pub margin: Decimal,

    #[serde(serialize_with = "serialize_decimal")]
    pub leverage: Decimal,

    #[serde(serialize_with = "serialize_decimal")]
    pub pnl: Decimal,

    #[serde(serialize_with = "serialize_decimal")]
    pub liquidation_price: Decimal,

    pub position_type: PositionType,
}
