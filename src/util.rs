use std::{collections::BTreeMap, marker::PhantomData, str::FromStr};

use serde::{
    de::{DeserializeOwned, IntoDeserializer},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use serde_with::{DeserializeAs, SerializeAs};

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

pub fn group_by_prefix<A, B, I>(v: I, delimiter: &str) -> BTreeMap<String, Vec<(String, String)>>
where
    A: AsRef<str>,
    B: AsRef<str>,
    I: IntoIterator<Item = (A, B)>,
{
    v.into_iter()
        .fold(BTreeMap::new(), |mut groups, (key, value)| {
            if let Some((prefix, key)) = key.as_ref().split_once(delimiter) {
                groups
                    .entry(prefix.to_owned())
                    .or_default()
                    .push((key.to_owned(), value.as_ref().to_owned()));
            }
            groups
        })
}

pub trait ResultLogger<F> {
    fn on_error_log(self, cb: F) -> Self;
}

impl<F, S, T, E> ResultLogger<F> for std::result::Result<T, E>
where
    S: AsRef<str>,
    F: FnOnce(&E) -> S,
{
    fn on_error_log(self, cb: F) -> Self {
        return self.map_err(|e| {
            println!("{}", cb(&e).as_ref());
            e
        });
    }
}

impl<F, S, T> ResultLogger<F> for Option<T>
where
    S: AsRef<str>,
    F: FnOnce() -> S,
{
    fn on_error_log(self, cb: F) -> Self {
        if self.is_none() {
            println!("{}", cb().as_ref())
        }
        self
    }
}
