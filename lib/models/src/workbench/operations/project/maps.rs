//! JSON represents structured map keys as key/value rows instead of object keys.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, hash::Hash};

/// Preserve keys that JSON objects cannot represent, such as grid coordinates.
pub fn serialize<K: Serialize, V: Serialize, S: Serializer>(
    map: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut rows = map
        .iter()
        .map(|(key, value)| serde_json::to_string(key).map(|order| (order, key, value)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(serde::ser::Error::custom)?;
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows.into_iter()
        .map(|(_, key, value)| (key, value))
        .collect::<Vec<_>>()
        .serialize(serializer)
}

/// Reject duplicate rows rather than silently replacing stored state.
pub fn deserialize<
    'de,
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    D: Deserializer<'de>,
>(
    deserializer: D,
) -> Result<HashMap<K, V>, D::Error> {
    let rows = Vec::<(K, V)>::deserialize(deserializer)?;
    let count = rows.len();
    let map: HashMap<_, _> = rows.into_iter().collect();
    if map.len() != count {
        return Err(serde::de::Error::custom("duplicate state key"));
    }
    Ok(map)
}
