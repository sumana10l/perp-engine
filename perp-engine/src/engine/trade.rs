use crate::engine::position::PositionType;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize, Serializer};

fn serialize_decimal<S>(d: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(d.to_f64().unwrap_or(0.0))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    #[serde(serialize_with = "serialize_decimal")]
    pub entry: Decimal,

    #[serde(serialize_with = "serialize_decimal")]
    pub exit: Decimal,

    #[serde(serialize_with = "serialize_decimal")]
    pub pnl: Decimal,

    pub position_type: PositionType,
}
