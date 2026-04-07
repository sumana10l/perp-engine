use crate::engine::position::PositionType;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};

fn serialize_decimal<S>(d: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&d.to_string())
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
    pub asset: String,
    pub closed_at: DateTime<Utc>,
}
