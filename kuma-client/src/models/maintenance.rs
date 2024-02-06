//! Models related to Uptime Kuma maintenances

use crate::deserialize::{
    DeserializeBoolLenient, DeserializeNumberLenient, SerializeDateRange, SerializeTimeRange,
};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_inline_default::serde_inline_default;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{serde_as, skip_serializing_none};
use std::{collections::HashMap, fmt};
use time::{PrimitiveDateTime, Time};

include!(concat!(env!("OUT_DIR"), "/timezones.rs"));

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceMonitor {
    #[serde(rename = "id")]
    pub id: Option<i32>,

    #[serde(rename = "pathName")]
    pub path_name: Option<String>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceStatusPage {
    #[serde(rename = "id")]
    pub id: Option<i32>,

    #[serde(rename = "name")]
    pub name: Option<String>,
}

#[derive(Clone, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 0,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DayOfMonth {
    Day(u8),
    LastDay,
}

impl<'de> Deserialize<'de> for DayOfMonth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        match value {
            Value::Number(n) if n.is_u64() => {
                Ok(DayOfMonth::Day(n.as_u64().unwrap().try_into().unwrap()))
            }
            Value::String(s) if s == "lastDay1" => Ok(DayOfMonth::LastDay),
            _ => Err(serde::de::Error::custom("Invalid DayOfMonth format")),
        }
    }
}

impl Serialize for DayOfMonth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DayOfMonth::Day(day) => serializer.serialize_u8(*day),
            DayOfMonth::LastDay => serializer.serialize_str("lastDay1"),
        }
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct TimeSlot {
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,

    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimeZoneOption {
    SameAsServer(Option<TimeZone>),
    UTC,
    TimeZone(TimeZone),
}

impl Serialize for TimeZoneOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (timezone, timezone_option, timezone_offset) = match self {
            TimeZoneOption::SameAsServer(tz) => (
                tz.as_ref()
                    .map(|tz| tz.identifier().to_owned())
                    .unwrap_or("UTC".to_owned()),
                "SAME_AS_SERVER".to_owned(),
                tz.as_ref()
                    .map(|tz| tz.utc_offset().to_owned())
                    .unwrap_or("+00:00".to_owned()),
            ),
            TimeZoneOption::UTC => ("UTC".to_owned(), "UTC".to_owned(), "+00:00".to_owned()),
            TimeZoneOption::TimeZone(timezone) => (
                timezone.identifier().to_owned(),
                timezone.identifier().to_owned(),
                timezone.utc_offset().to_owned(),
            ),
        };

        let mut ser_struct = serializer.serialize_struct("TimeZone", 3)?;
        ser_struct.serialize_field("timezone", &timezone)?;
        ser_struct.serialize_field("timezoneOption", &timezone_option)?;
        ser_struct.serialize_field("timezoneOffset", &timezone_offset)?;
        ser_struct.end()
    }
}

impl<'de> Deserialize<'de> for TimeZoneOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            TimeZone,
            TimeZoneOption,
            TimeZoneOffset,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`timezone`, `timezoneOption` or `timezoneOffset`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "timezone" => Ok(Field::TimeZone),
                            "timezoneOption" => Ok(Field::TimeZoneOption),
                            "timezoneOffset" => Ok(Field::TimeZoneOffset),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct TimeZoneOptionVisitor;

        impl<'de> Visitor<'de> for TimeZoneOptionVisitor {
            type Value = TimeZoneOption;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct TimeZoneOption")
            }

            fn visit_map<V>(self, mut map: V) -> Result<TimeZoneOption, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut timezone_identifier: Option<String> = None;
                let mut timezone_option: Option<String> = None;
                let mut timezone_offset: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::TimeZone => {
                            if timezone_identifier.is_some() {
                                return Err(de::Error::duplicate_field("timezone"));
                            }
                            timezone_identifier = Some(map.next_value()?);
                        }
                        Field::TimeZoneOption => {
                            if timezone_option.is_some() {
                                return Err(de::Error::duplicate_field("timezoneOption"));
                            }
                            timezone_option = Some(map.next_value()?);
                        }
                        Field::TimeZoneOffset => {
                            if timezone_offset.is_some() {
                                return Err(de::Error::duplicate_field("timezoneOffset"));
                            }
                            timezone_offset = Some(map.next_value()?);
                        }
                    }
                }
                let timezone_identifier =
                    timezone_identifier.ok_or_else(|| de::Error::missing_field("timezone"))?;
                let timezone_option =
                    timezone_option.ok_or_else(|| de::Error::missing_field("timezoneOption"))?;
                let _timezone_offset =
                    timezone_offset.ok_or_else(|| de::Error::missing_field("timezoneOffset"))?;

                let timezone = TimeZone::from_str(&timezone_identifier).ok_or_else(|| {
                    de::Error::invalid_value(
                        de::Unexpected::Str(&timezone_identifier),
                        &"a valid timezone identifier",
                    )
                })?;

                Ok(match timezone_option.as_ref() {
                    "SAME_AS_SERVER" => TimeZoneOption::SameAsServer(Some(timezone)),
                    "UTC" => TimeZoneOption::UTC,
                    _ => TimeZoneOption::TimeZone(timezone),
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["timezone", "timezoneOption", "timezoneOffset"];
        deserializer.deserialize_struct("TimeZoneOption", FIELDS, TimeZoneOptionVisitor)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceCommon {
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    #[serde(rename = "title")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "active")]
    #[serde_inline_default(Some(true))]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub active: Option<bool>,

    #[serde(rename = "status")]
    pub status: Option<String>,

    #[serde(rename = "monitors")]
    #[serde(default)]
    pub monitors: Option<Vec<MaintenanceMonitor>>,

    #[serde(rename = "statusPages")]
    #[serde(default)]
    pub status_pages: Option<Vec<MaintenanceStatusPage>>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceSchedule {
    #[serde(rename = "dateRange")]
    #[serde_as(as = "Option<SerializeDateRange>")]
    pub date_range: Option<Range<PrimitiveDateTime>>,

    #[serde(rename = "timeRange")]
    #[serde_as(as = "Option<SerializeTimeRange>")]
    pub time_range: Option<Range<Time>>,

    #[serde(flatten)]
    #[serde(rename = "timezone")]
    pub timezone: Option<TimeZoneOption>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceCron {
    #[serde(rename = "cron")]
    #[serde_inline_default(Some("30 3 * * *".to_owned()))]
    pub cron: Option<String>,

    #[serde(rename = "durationMinutes")]
    #[serde_inline_default(Some(60.0))]
    pub duration_minutes: Option<f64>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceRecurringInterval {
    #[serde(rename = "intervalDay")]
    pub interval: Option<u8>,

    #[serde(rename = "timeslotList")]
    #[serde(default)]
    pub timeslots: Vec<TimeSlot>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceRecurringWeekday {
    #[serde(rename = "timeslotList")]
    #[serde(default)]
    pub timeslots: Vec<TimeSlot>,

    #[serde(rename = "weekdays")]
    #[serde(default)]
    pub weekdays: Vec<Weekday>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceRecurringDayOfMonth {
    #[serde(rename = "daysOfMonth")]
    #[serde(default)]
    pub days_of_month: Vec<DayOfMonth>,

    #[serde(rename = "timeslotList")]
    #[serde(default)]
    pub timeslots: Vec<TimeSlot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "strategy")]
pub enum Maintenance {
    #[serde(rename = "manual")]
    Manual {
        #[serde(flatten)]
        common: MaintenanceCommon,
    },

    #[serde(rename = "single")]
    Single {
        #[serde(flatten)]
        common: MaintenanceCommon,
        #[serde(flatten)]
        schedule: MaintenanceSchedule,
    },

    #[serde(rename = "cron")]
    Cron {
        #[serde(flatten)]
        common: MaintenanceCommon,
        #[serde(flatten)]
        schedule: MaintenanceSchedule,
        #[serde(flatten)]
        cron: MaintenanceCron,
    },

    #[serde(rename = "recurring-interval")]
    RecurringInterval {
        #[serde(flatten)]
        common: MaintenanceCommon,
        #[serde(flatten)]
        schedule: MaintenanceSchedule,
        #[serde(flatten)]
        recurring_interval: MaintenanceRecurringInterval,
    },

    #[serde(rename = "recurring-weekday")]
    RecurringWeekday {
        #[serde(flatten)]
        common: MaintenanceCommon,
        #[serde(flatten)]
        schedule: MaintenanceSchedule,
        #[serde(flatten)]
        recurring_weekday: MaintenanceRecurringWeekday,
    },

    #[serde(rename = "recurring-day-of-month")]
    RecurringDayOfMonth {
        #[serde(flatten)]
        common: MaintenanceCommon,
        #[serde(flatten)]
        schedule: MaintenanceSchedule,
        #[serde(flatten)]
        recurring_day_of_month: MaintenanceRecurringDayOfMonth,
    },
}

impl Maintenance {
    pub fn common(&self) -> &MaintenanceCommon {
        match self {
            Maintenance::Manual { common } => common,
            Maintenance::Single { common, .. } => common,
            Maintenance::Cron { common, .. } => common,
            Maintenance::RecurringInterval { common, .. } => common,
            Maintenance::RecurringWeekday { common, .. } => common,
            Maintenance::RecurringDayOfMonth { common, .. } => common,
        }
    }
    pub fn common_mut(&mut self) -> &mut MaintenanceCommon {
        match self {
            Maintenance::Manual { common } => common,
            Maintenance::Single { common, .. } => common,
            Maintenance::Cron { common, .. } => common,
            Maintenance::RecurringInterval { common, .. } => common,
            Maintenance::RecurringWeekday { common, .. } => common,
            Maintenance::RecurringDayOfMonth { common, .. } => common,
        }
    }
}

pub type MaintenanceList = HashMap<String, Maintenance>;
