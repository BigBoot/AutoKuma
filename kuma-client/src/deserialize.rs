use crate::maintenance::Range;
use serde::{
    de::{DeserializeOwned, IntoDeserializer},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use serde_with::{DeserializeAs, SerializeAs};
use std::{collections::HashMap, hash::Hash, marker::PhantomData, str::FromStr};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime, Time};

pub(crate) struct DeserializeNumberLenient;

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

pub(crate) struct DeserializeBoolLenient;

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
            Value::Number(n) => match (n.as_f64(), n.as_i64(), n.as_u64()) {
                (Some(n), _, _) => Ok(n != 0.0),
                (_, Some(n), _) => Ok(n != 0),
                (_, _, Some(n)) => Ok(n != 0),
                _ => unreachable!(),
            },
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

pub(crate) struct DeserializeVecLenient<T>(PhantomData<T>);

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

pub(crate) struct DeserializeHashMapLenient<K, V>(PhantomData<K>, PhantomData<V>);

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

pub(crate) struct DeserializeValueLenient;

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

pub(crate) struct SerializeDateTime;

impl<'de> DeserializeAs<'de, PrimitiveDateTime> for DeserializeBoolLenient {
    fn deserialize_as<D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        PrimitiveDateTime::parse(&String::deserialize(deserializer)?, &Iso8601::DATE_TIME)
            .map_err(serde::de::Error::custom)
    }
}

impl SerializeAs<PrimitiveDateTime> for SerializeDateTime {
    fn serialize_as<S>(source: &PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        source
            .format(&Iso8601::DATE_TIME)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
pub(crate) struct SerializeDateRange;

impl<'de> DeserializeAs<'de, Range<PrimitiveDateTime>> for SerializeDateRange {
    fn deserialize_as<D>(deserializer: D) -> Result<Range<PrimitiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Vec::<String>::deserialize(deserializer)
            .map_err(serde::de::Error::custom)?
            .into_iter()
            .map(|s| PrimitiveDateTime::parse(&s, &Iso8601::DATE_TIME))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)?;

        if value.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "Expected array of length 2 but got array of length {}",
                value.len()
            )));
        };

        let mut iter = value.into_iter();
        Ok(Range {
            start: iter.next().unwrap(),
            end: iter.next().unwrap(),
        })
    }
}

impl SerializeAs<Range<PrimitiveDateTime>> for SerializeDateRange {
    fn serialize_as<S>(source: &Range<PrimitiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        vec![
            &source
                .start
                .format(&Iso8601::DATE_TIME)
                .map_err(serde::ser::Error::custom)?,
            &source
                .end
                .format(&Iso8601::DATE_TIME)
                .map_err(serde::ser::Error::custom)?,
        ]
        .serialize(serializer)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TimePoint {
    pub(crate) hours: u8,
    pub(crate) minutes: u8,
    pub(crate) seconds: u8,
}

pub(crate) struct SerializeTimeRange;

impl<'de> DeserializeAs<'de, Range<Time>> for SerializeTimeRange {
    fn deserialize_as<D>(deserializer: D) -> Result<Range<Time>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Vec::<TimePoint>::deserialize(deserializer)?
            .into_iter()
            .map(|time| Time::from_hms(time.hours, time.minutes, time.seconds))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)?;

        if value.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "Expected array of length 2 but got array of length {}",
                value.len()
            )));
        };

        let mut iter = value.into_iter();
        Ok(Range {
            start: iter.next().unwrap(),
            end: iter.next().unwrap(),
        })
    }
}

impl SerializeAs<Range<Time>> for SerializeTimeRange {
    fn serialize_as<S>(source: &Range<Time>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        for t in [&source.start, &source.end] {
            seq.serialize_element(&TimePoint {
                hours: t.hour(),
                minutes: t.minute(),
                seconds: t.second(),
            })?;
        }
        seq.end()
    }
}
