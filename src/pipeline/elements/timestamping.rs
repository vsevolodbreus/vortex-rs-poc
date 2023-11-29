//! Timestamping Pipeline Element
use std::fmt::Display;

use chrono::{DateTime, Local, SecondsFormat, TimeZone, Utc};
use serde_json::Value;

use crate::crawler::Item;
use crate::pipeline::elements::PipelineElement;
use crate::settings::TimestampingSettings;

/// The time offsets available for Timestamping
#[derive(Clone, Debug, Deserialize)]
pub enum TimeOffset {
    /// The system local time zone
    Local,

    /// The UTC time zone
    Utc,
}

/// Various time formats that Timestamping uses
pub enum TimeFormat {
    /// Format according to RFC 2822.
    /// Example: "Wed,  9 Jan 2019 20:05:56 -0800"
    Rfc2822,

    /// Format according to RFC 3339 and ISO 8601.
    /// Example: "2019-01-09T20:05:56-08:00"
    Rfc3339,

    /// Formats the combined date and time with the specified format string.
    /// See the chrono::format::strftime module on the supported escape sequences.
    /// Example: "%D %H:%M:%S" -> "01/09/19 20:20:17"
    Format(String),

    /// The number of non-leap-seconds since January 1, 1970 0:00:00 UTC (aka "UNIX timestamp").
    /// Example: "1547094087"
    Timestamp,

    /// The number of non-leap-milliseconds since January 1, 1970 UTC.
    /// Example: "1547094141733"
    TimestampMs,
}

/// Pipeline Element that provides a way to configure custom timestamping behavior on `Item`s
pub struct Timestamping {
    offset: TimeOffset,
    format: TimeFormat,
    field: String,
}

impl Default for Timestamping {
    fn default() -> Self {
        Timestamping::new(TimeOffset::Utc, TimeFormat::Timestamp)
    }
}

impl Timestamping {
    pub fn new(offset: TimeOffset, format: TimeFormat) -> Self {
        Self { offset, format, field: "timestamp".to_string() }
    }

    pub fn with_format(format: TimeFormat) -> Self {
        Timestamping::new(TimeOffset::Utc, format)
    }

    pub fn with_offset(offset: TimeOffset) -> Self {
        Timestamping::new(offset, TimeFormat::Timestamp)
    }

    pub fn from_settings(settings: TimestampingSettings) -> Self {
        let format = match settings.format.as_str() {
            "Rfc2822" => TimeFormat::Rfc2822,
            "Rfc3339" => TimeFormat::Rfc3339,
            "Timestamp" => TimeFormat::Timestamp,
            "TimestampMs" => TimeFormat::TimestampMs,
            frm => TimeFormat::Format(frm.to_string()),
        };

        Self {
            offset: settings.offset,
            format,
            field: settings.field,
        }
    }

    pub fn set_field(&mut self, name: &str) {
        self.field = name.to_string();
    }
}

impl PipelineElement for Timestamping {
    fn process_item(&self, mut item: Item) -> Item {
        if let Some(data) = item.data.as_object_mut() {
            let v = match self.offset {
                TimeOffset::Local => Utils::convert::<Local>(Local::now(), &self.format),
                TimeOffset::Utc => Utils::convert::<Utc>(Utc::now(), &self.format),
            };
            data.insert(self.field.to_string(), Value::String(v));
        }
        item
    }
}

struct Utils;

impl Utils {
    fn convert<T>(dt: DateTime<T>, format: &TimeFormat) -> String
        where T: TimeZone,
              T::Offset: Display,
    {
        match format {
            TimeFormat::Rfc2822 => dt.to_rfc2822(),
            TimeFormat::Rfc3339 => dt.to_rfc3339_opts(SecondsFormat::Secs, false),
            TimeFormat::Format(frm) => format!("{}", dt.format(frm)),
            TimeFormat::Timestamp => dt.timestamp().to_string(),
            TimeFormat::TimestampMs => dt.timestamp_millis().to_string(),
        }
    }
}
