//! Timestamp generation processor.
//!
//! `qed:timestamp()` generates a formatted timestamp string. The input is
//! ignored entirely — output is produced purely from parameters.

use chrono::{DateTime, FixedOffset, Utc};

use super::{Processor, ProcessorError};

/// The output format for the timestamp.
#[derive(Debug)]
pub(crate) enum TimestampFormat {
    /// `2026-02-28T13:45:00Z`
    Iso8601,
    /// Unix epoch seconds: `1740749100`
    Unix,
    /// Unix epoch milliseconds: `1740749100000`
    UnixMs,
    /// `2026-02-28`
    Date,
    /// `13:45:00`
    Time,
    /// `2026-02-28 13:45:00`
    DateTime,
    /// User-supplied strftime format string.
    Custom(String),
}

/// Which timezone to render the timestamp in.
#[derive(Debug)]
pub(crate) enum Timezone {
    Utc,
    Iana(chrono_tz::Tz),
    Fixed(FixedOffset),
}

// @spec GEN-001, GEN-002, GEN-003, GEN-004, GEN-020, GEN-021, GEN-022, GEN-023, GEN-024, GEN-025, GEN-026, GEN-027
/// Generates a formatted timestamp string.
#[derive(Debug)]
pub(crate) struct TimestampProcessor {
    pub(crate) format: TimestampFormat,
    pub(crate) timezone: Timezone,
}

impl Processor for TimestampProcessor {
    fn execute(&self, _input: &str) -> Result<String, ProcessorError> {
        let now = Utc::now();
        let formatted = match &self.timezone {
            Timezone::Utc => format_utc(now, &self.format),
            Timezone::Iana(tz) => format_offset(now.with_timezone(tz).fixed_offset(), &self.format),
            Timezone::Fixed(offset) => format_offset(now.with_timezone(offset), &self.format),
        };
        let mut out = formatted;
        out.push('\n');
        Ok(out)
    }
}

/// Format a UTC timestamp.
fn format_utc(now: DateTime<Utc>, fmt: &TimestampFormat) -> String {
    match fmt {
        TimestampFormat::Iso8601 => now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        TimestampFormat::Unix => now.timestamp().to_string(),
        TimestampFormat::UnixMs => now.timestamp_millis().to_string(),
        TimestampFormat::Date => now.format("%Y-%m-%d").to_string(),
        TimestampFormat::Time => now.format("%H:%M:%S").to_string(),
        TimestampFormat::DateTime => now.format("%Y-%m-%d %H:%M:%S").to_string(),
        TimestampFormat::Custom(s) => now.format(s).to_string(),
    }
}

/// Format a timestamp with a fixed offset (from IANA or explicit offset).
fn format_offset(dt: DateTime<FixedOffset>, fmt: &TimestampFormat) -> String {
    match fmt {
        TimestampFormat::Iso8601 => dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        TimestampFormat::Unix => dt.timestamp().to_string(),
        TimestampFormat::UnixMs => dt.timestamp_millis().to_string(),
        TimestampFormat::Date => dt.format("%Y-%m-%d").to_string(),
        TimestampFormat::Time => dt.format("%H:%M:%S").to_string(),
        TimestampFormat::DateTime => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        TimestampFormat::Custom(s) => dt.format(s).to_string(),
    }
}

// @spec GEN-028, GEN-029, GEN-030
/// Translate an LDML-style format string to a chrono strftime string.
///
/// Supported tokens:
/// - `yyyy` → `%Y` (4-digit year)
/// - `MM` → `%m` (2-digit month)
/// - `dd` → `%d` (2-digit day)
/// - `HH` → `%H` (2-digit hour, 24h)
/// - `mm` → `%M` (2-digit minute)
/// - `ss` → `%S` (2-digit second)
///
/// All other characters pass through as literals.
pub(crate) fn ldml_to_strftime(ldml: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = ldml.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 4 <= len && chars[i..i + 4] == ['y', 'y', 'y', 'y'] {
            result.push_str("%Y");
            i += 4;
        } else if i + 2 <= len && chars[i] == 'M' && chars[i + 1] == 'M' {
            result.push_str("%m");
            i += 2;
        } else if i + 2 <= len && chars[i] == 'd' && chars[i + 1] == 'd' {
            result.push_str("%d");
            i += 2;
        } else if i + 2 <= len && chars[i] == 'H' && chars[i + 1] == 'H' {
            result.push_str("%H");
            i += 2;
        } else if i + 2 <= len && chars[i] == 'm' && chars[i + 1] == 'm' {
            result.push_str("%M");
            i += 2;
        } else if i + 2 <= len && chars[i] == 's' && chars[i + 1] == 's' {
            result.push_str("%S");
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Parse a fixed-offset timezone string like `"UTC+5:30"` or `"UTC-8"`.
///
/// Returns `None` if the string doesn't match the expected format.
pub(crate) fn parse_fixed_offset(s: &str) -> Option<FixedOffset> {
    let rest = s.strip_prefix("UTC")?;
    if rest.is_empty() {
        return FixedOffset::east_opt(0);
    }

    let (sign, digits) = if let Some(d) = rest.strip_prefix('+') {
        (1, d)
    } else if let Some(d) = rest.strip_prefix('-') {
        (-1, d)
    } else {
        return None;
    };

    let (hours, minutes) = if let Some((h, m)) = digits.split_once(':') {
        (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?)
    } else {
        (digits.parse::<i32>().ok()?, 0)
    };

    let total_seconds = sign * (hours * 3600 + minutes * 60);
    FixedOffset::east_opt(total_seconds)
}
