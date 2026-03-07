use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "move_up")]
    MoveUp,
    #[serde(rename = "move_down")]
    MoveDown,
    #[serde(rename = "move_left")]
    MoveLeft,
    #[serde(rename = "move_right")]
    MoveRight,
    #[serde(rename = "place_bomb")]
    PlaceBomb,
    #[serde(rename = "nothing")]
    DoNothing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Air,
    Wall,
    Box,
}

impl Serialize for CellType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CellType::Air => serializer.serialize_str("AIR"),
            CellType::Wall => serializer.serialize_str("WALL"),
            CellType::Box => serializer.serialize_str("BOX"),
        }
    }
}

impl<'de> Deserialize<'de> for CellType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::String(s) => match s.as_str() {
                "WALL" => Ok(CellType::Wall),
                "BOX" => Ok(CellType::Box),
                _ => Ok(CellType::Air),
            },
            serde_json::Value::Number(n) => {
                if let Some(n) = n.as_i64() {
                    match n {
                        0 => Ok(CellType::Wall),
                        2 => Ok(CellType::Box),
                        _ => Ok(CellType::Air),
                    }
                } else {
                    Err(D::Error::custom("unsupported cell encoding"))
                }
            }
            _ => Err(D::Error::custom("unsupported cell encoding")),
        }
    }
}
