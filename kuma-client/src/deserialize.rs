use serde::{
    de::{DeserializeOwned, IntoDeserializer},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use serde_with::{DeserializeAs, SerializeAs};
use std::{collections::HashMap, hash::Hash, marker::PhantomData, str::FromStr};

pub struct DeserializeNumberLenient;

impl<'de, T> DeserializeAs<'de, T> for DeserializeNumberLenient
where
    T: FromStr + TryFrom<i64>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let result = match value {
            Value::Number(n) => Ok(n.as_i64().and_then(|n| n.try_into().ok()).ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "Unable to represent {} as {}",
                    n,
                    std::any::type_name::<T>()
                ))
            }))?,
            Value::String(s) => s.parse::<T>().map_err(|_| {
                serde::de::Error::custom(format!(
                    "Unable to parse {} as {}",
                    s,
                    std::any::type_name::<T>()
                ))
            }),
            _ => Err(serde::de::Error::custom(
                "Unexpected type for deserialization",
            )),
        };

        result
    }
}

impl<T> SerializeAs<T> for DeserializeNumberLenient
where
    T: Serialize,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        source.serialize(serializer)
    }
}

pub struct DeserializeBoolLenient;

impl<'de> DeserializeAs<'de, bool> for DeserializeBoolLenient {
    fn deserialize_as<D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let result = match value {
            Value::Bool(b) => Ok(b),
            Value::String(s) => s.to_lowercase().parse::<bool>().map_err(|_| {
                serde::de::Error::custom(format!(
                    "Unable to parse {} as {}",
                    s,
                    std::any::type_name::<bool>()
                ))
            }),
            _ => Err(serde::de::Error::custom(
                "Unexpected type for deserialization",
            )),
        };

        result
    }
}

impl<T> SerializeAs<T> for DeserializeBoolLenient
where
    T: Serialize,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        source.serialize(serializer)
    }
}

pub struct DeserializeVecLenient<T>(PhantomData<T>);

impl<'de, T> DeserializeAs<'de, Vec<T>> for DeserializeVecLenient<T>
where
    T: DeserializeOwned,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)
            .map_err(serde::de::Error::custom)?
            .clone();

        return match value {
            Value::Array(_) => {
                Vec::<T>::deserialize(value.into_deserializer()).map_err(serde::de::Error::custom)
            }
            Value::String(s) => serde_json::from_str(&s).map_err(serde::de::Error::custom),
            _ => Err(serde::de::Error::custom(
                "Unexpected type for deserialization",
            )),
        };
    }
}

impl<T> SerializeAs<Vec<T>> for DeserializeVecLenient<T>
where
    T: Serialize,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        source.serialize(serializer)
    }
}

pub struct DeserializeHashMapLenient<K, V>(PhantomData<K>, PhantomData<V>);

impl<'de, K, V> DeserializeAs<'de, HashMap<K, V>> for DeserializeHashMapLenient<K, V>
where
    K: DeserializeOwned + Eq + Hash,
    V: DeserializeOwned,
{
    fn deserialize_as<D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)
            .map_err(serde::de::Error::custom)?
            .clone();

        return match value {
            Value::Object(_) => HashMap::<K, V>::deserialize(value.into_deserializer())
                .map_err(serde::de::Error::custom),
            Value::String(s) => serde_json::from_str(&s).map_err(serde::de::Error::custom),
            _ => Err(serde::de::Error::custom(
                "Unexpected type for deserialization",
            )),
        };
    }
}

impl<K, V> SerializeAs<HashMap<K, V>> for DeserializeHashMapLenient<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize_as<S>(source: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        source.serialize(serializer)
    }
}

pub struct DeserializeValueLenient;

impl<'de> DeserializeAs<'de, Value> for DeserializeValueLenient {
    fn deserialize_as<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let result = match value {
            Value::String(s) => s.parse::<Value>().map_err(|_| {
                serde::de::Error::custom(format!(
                    "Unable to parse {} as {}",
                    s,
                    std::any::type_name::<Value>()
                ))
            }),
            value => Ok(value),
        };

        result
    }
}

impl<T> SerializeAs<T> for DeserializeValueLenient
where
    T: Serialize,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        source.serialize(serializer)
    }
}
