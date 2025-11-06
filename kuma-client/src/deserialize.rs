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

impl<'de> DeserializeAs<'de, PrimitiveDateTime> for DeserializeBoolLenient {
    fn deserialize_as<D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        PrimitiveDateTime::parse(&String::deserialize(deserializer)?, &Iso8601::DATE_TIME)
            .map_err(serde::de::Error::custom)
    }
}

pub struct SerializeDateRange;

impl<'de> DeserializeAs<'de, Option<Range<PrimitiveDateTime>>> for SerializeDateRange {
    fn deserialize_as<D>(deserializer: D) -> Result<Option<Range<PrimitiveDateTime>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(source) = Option::<Vec<Option<String>>>::deserialize(deserializer)? {
            let value = source
                .into_iter()
                .map(|o| {
                    o.map(|s| -> Result<PrimitiveDateTime, D::Error> {
                        if let Ok(dt) = dateparser::parse(&s) {
                            return Ok(PrimitiveDateTime::parse(
                                &dt.to_rfc3339(),
                                &Iso8601::DATE_TIME,
                            )
                            .unwrap());
                        }
                        if let Ok(dt) = PrimitiveDateTime::parse(&s, &Iso8601::DATE_TIME) {
                            return Ok(dt);
                        }
                        Err(serde::de::Error::custom(format!(
                            "Unable to parse {} as DateTime",
                            s
                        )))
                    })
                    .transpose()
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(serde::de::Error::custom)?;

            match value.len() {
                1 if value[0].is_none() => Ok(None),
                2 => {
                    let mut iter = value.into_iter();
                    Ok(Some(Range {
                        start: iter.next().unwrap().unwrap(),
                        end: iter.next().unwrap().unwrap(),
                    }))
                }
                _ => Err(serde::de::Error::custom(format!(
                "Expected DateRange to be [Null] or array of length 2 but got array of length {}",
                value.len()
            ))),
            }
        } else {
            Ok(None)
        }
    }
}

impl SerializeAs<Option<Range<PrimitiveDateTime>>> for SerializeDateRange {
    fn serialize_as<S>(
        source: &Option<Range<PrimitiveDateTime>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let values = match &source {
            None => vec![None],
            Some(Range { start, end }) => vec![
                Some(
                    start
                        .format(&Iso8601::DATE_TIME)
                        .map_err(serde::ser::Error::custom)?,
                ),
                Some(
                    end.format(&Iso8601::DATE_TIME)
                        .map_err(serde::ser::Error::custom)?,
                ),
            ],
        };

        values.serialize(serializer)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimePoint {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: Option<u8>,
}

pub struct SerializeTimeRange;

impl<'de> DeserializeAs<'de, Range<Time>> for SerializeTimeRange {
    fn deserialize_as<D>(deserializer: D) -> Result<Range<Time>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Vec::<TimePoint>::deserialize(deserializer)?
            .into_iter()
            .map(|time| Time::from_hms(time.hours, time.minutes, time.seconds.unwrap_or_default()))
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
                seconds: Some(t.second()),
            })?;
        }
        seq.end()
    }
}
