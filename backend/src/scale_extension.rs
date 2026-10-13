pub mod scale_serde {
    use std::str::FromStr;

    use harmonia::ScaleKind;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(scale: &ScaleKind, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&scale.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ScaleKind, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match ScaleKind::from_str(s.as_str()) {
            Ok(sc) => Ok(sc),
            Err(_) => Err(serde::de::Error::custom(format!("unknown note name: {s}"))),
        }
    }
}
