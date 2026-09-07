//! Foundational domain primitives.
//!
//! Pure value types with serde + Display. These are intentionally dependency-light
//! and deterministic (RFC 3339 UTC timestamps, validated semver/language/confidence).

use core::fmt;
use std::str::FromStr;

/// A parse error for a UUID-backed newtype ID. Carries a stable error code so the
/// diagnostic layer (D0.3) can map it consistently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UuidParseError;

impl fmt::Display for UuidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QAI-DOM-0001: expected a UUID string")
    }
}

impl std::error::Error for UuidParseError {}

/// Semantic version per https://semver.org (major.minor.patch).
///
/// Not a full semver-range parser; only the canonical `MAJOR.MINOR.PATCH` form is
/// supported, which is all the domain model needs for parser/tool/edition versions
/// (PRD §76). Comparisons follow numerical precedence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
}

/// A malformed semantic-version string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemVerParseError;

impl fmt::Display for SemVerParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QAI-DOM-0002: expected semantic version `MAJOR.MINOR.PATCH`")
    }
}

impl std::error::Error for SemVerParseError {}

impl SemVer {
    /// Constructs a `SemVer` from parts.
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { major, minor, patch }
    }

    /// The major component.
    pub const fn major(&self) -> u64 {
        self.major
    }
    /// The minor component.
    pub const fn minor(&self) -> u64 {
        self.minor
    }
    /// The patch component.
    pub const fn patch(&self) -> u64 {
        self.patch
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for SemVer {
    type Err = SemVerParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        // Require exactly 3 numeric parts, each non-empty and parseable.
        let major = parse_numeric(parts.next())?;
        let minor = parse_numeric(parts.next())?;
        let patch = parse_numeric(parts.next())?;
        // Reject trailing content like "1.2.3.4".
        if parts.next().is_some() {
            return Err(SemVerParseError);
        }
        Ok(Self::new(major, minor, patch))
    }
}

fn parse_numeric(s: Option<&str>) -> Result<u64, SemVerParseError> {
    match s {
        Some(digits) if !digits.is_empty() => digits
            .parse()
            .map_err(|_| SemVerParseError),
        _ => Err(SemVerParseError),
    }
}

impl serde::Serialize for SemVer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SemVer {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A UTC timestamp in RFC 3339 format with microsecond precision (PRD §82, §76).
///
/// Deterministic ordering for indexing/sorting is handled by the storage layer
/// (stored alongside an epoch-microsecond column); this type is the domain-facing
/// value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(time::OffsetDateTime);

/// A malformed timestamp string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimestampParseError;

impl fmt::Display for TimestampParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QAI-DOM-0003: expected RFC 3339 UTC timestamp (e.g. 2026-01-01T00:00:00Z)"
        )
    }
}

impl std::error::Error for TimestampParseError {}

impl Timestamp {
    /// The current UTC time with microsecond precision.
    pub fn now() -> Self {
        let now = time::OffsetDateTime::now_utc();
        Self(now.replace_nanosecond(
            now.nanosecond() / 1_000 * 1_000,
        ).unwrap_or(now))
    }

    /// Construct from explicit date/time parts.
    pub fn from_ymd_hms(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, TimestampParseError> {
        let dt = time::Date::from_calendar_date(year, month.try_into().map_err(|_| TimestampParseError)?, day)
            .map_err(|_| TimestampParseError)?
            .with_hms(hour, minute, second)
            .map_err(|_| TimestampParseError)?
            .assume_utc();
        Ok(Self(dt))
    }

    /// The inner `OffsetDateTime`.
    pub const fn as_time(&self) -> time::OffsetDateTime {
        self.0
    }

    /// Epoch microseconds (UTC), for storage/indexing.
    pub fn epoch_micros(&self) -> i128 {
        self.0.unix_timestamp_nanos() / 1_000
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // RFC 3339 with microseconds; if nanosecond precision exists it is truncated earlier.
        write!(f, "{}", self.0.format(&time::format_description::well_known::Rfc3339).map_err(|_| fmt::Error)?)
    }
}

impl FromStr for Timestamp {
    type Err = TimestampParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dt = time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
            .map_err(|_| TimestampParseError)?;
        if dt.offset() != time::UtcOffset::UTC {
            return Err(TimestampParseError); // domain requires UTC
        }
        Ok(Self(dt))
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A BCP-47 language tag (e.g. `en`, `ar`, `fa-IR`). Validated loosely: the tag is
/// non-empty and contains only ASCII alphanumerics plus `-` — full BCP-47 grammar is
/// Phase 2 concern. PRD §6/§22 use language for editions and sources.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language(String);

/// A malformed language tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanguageParseError;

impl fmt::Display for LanguageParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QAI-DOM-0004: expected a BCP-47 language tag")
    }
}

impl std::error::Error for LanguageParseError {}

impl Language {
    /// The tag as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Language {
    type Err = LanguageParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valid = !s.is_empty()
            && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && s.as_bytes()[0] != b'-'
            && s.ends_with(|c: char| c.is_ascii_alphanumeric());
        if valid {
            Ok(Self(s.to_owned()))
        } else {
            Err(LanguageParseError)
        }
    }
}

impl serde::Serialize for Language {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Language {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A confidence score in the closed range `0.0 ..= 1.0` (PRD §6.4, §23).
///
/// Enforces bounds at construction so no later code can hold an out-of-range value.
/// Equality uses exact float comparison, which is fine because the value is
/// constructed exclusively from the validated range (no arithmetic is exposed).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Confidence(f32);

/// A confidence value outside `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfidenceOutOfRange;

impl fmt::Display for ConfidenceOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QAI-DOM-0005: confidence must be in 0.0..=1.0")
    }
}

impl std::error::Error for ConfidenceOutOfRange {}

impl Confidence {
    /// Validate a raw `f32` into a `Confidence`, rejecting out-of-range / NaN.
    pub fn new(value: f32) -> Result<Self, ConfidenceOutOfRange> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ConfidenceOutOfRange)
        }
    }

    /// The value as `f32`.
    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self(1.0)
    }
}

impl FromStr for Confidence {
    type Err = ConfidenceOutOfRange;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed: f32 = s.parse().map_err(|_| ConfidenceOutOfRange)?;
        Self::new(parsed)
    }
}

impl serde::Serialize for Confidence {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Confidence {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = f32::deserialize(deserializer)?;
        Self::new(v).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_parses_and_orders() {
        assert_eq!("1.2.3".parse::<SemVer>().unwrap(), SemVer::new(1, 2, 3));
        assert!("1.2".parse::<SemVer>().is_err());
        assert!("1.2.3.4".parse::<SemVer>().is_err());
        assert!("a.b.c".parse::<SemVer>().is_err());
        assert!(SemVer::new(2, 0, 0) > SemVer::new(1, 99, 99));
        assert_eq!(SemVer::new(1, 0, 0).to_string(), "1.0.0");
    }

    #[test]
    fn timestamp_rfc3339_roundtrip() {
        let ts = Timestamp::from_ymd_hms(2026, 1, 15, 10, 30, 0).unwrap();
        let s = ts.to_string();
        assert!(s.ends_with('Z'), "expected UTC, got {s}");
        let parsed: Timestamp = s.parse().unwrap();
        assert_eq!(parsed, ts);
    }

    #[test]
    fn timestamp_rejects_non_utc() {
        // "+02:00" offset is rejected (domain requires UTC).
        assert!("2026-01-15T10:30:00+02:00".parse::<Timestamp>().is_err());
        assert!("not-a-time".parse::<Timestamp>().is_err());
    }

    #[test]
    fn language_validates() {
        assert_eq!("en".parse::<Language>().unwrap().as_str(), "en");
        assert!("fa-IR".parse::<Language>().unwrap().as_str() == "fa-IR");
        assert!("".parse::<Language>().is_err());
        assert!("-en".parse::<Language>().is_err());
        assert!("en-".parse::<Language>().is_err());
        assert!("en_GB".parse::<Language>().is_err());
    }

    #[test]
    fn confidence_bounds() {
        assert_eq!(Confidence::new(0.0).unwrap().value(), 0.0);
        assert_eq!(Confidence::new(1.0).unwrap().value(), 1.0);
        assert!(Confidence::new(1.0001).is_err());
        assert!(Confidence::new(-0.1).is_err());
        assert!(Confidence::new(f32::NAN).is_err());
    }

    #[test]
    fn serialization_shapes() {
        assert_eq!(serde_json::to_string(&SemVer::new(1, 0, 0)).unwrap(), "\"1.0.0\"");
        let c = Confidence::new(0.5).unwrap();
        assert_eq!(serde_json::to_string(&c).unwrap(), "0.5");
        let back: Confidence = serde_json::from_str("0.5").unwrap();
        assert_eq!(back, c);
        // out-of-range rejects on deserialize
        assert!(serde_json::from_str::<Confidence>("1.5").is_err());
    }
}
