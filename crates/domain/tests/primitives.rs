//! Property-based tests for the domain primitives (P0-T05, PRD §55).
//!
//! These assert invariants that hold for *any* valid input, complementing the
//! hand-written unit tests in `src/primitives.rs`.

use domain::{Confidence, Language, SemVer, SourceId, Timestamp};
use proptest::prelude::*;

prop_compose! {
    /// A valid `MAJOR.MINOR.PATCH` semver string.
    fn semver_str()(major in 0u64..10_000, minor in 0u64..10_000, patch in 0u64..10_000)
        -> String {
        format!("{major}.{minor}.{patch}")
    }
}

proptest! {
    /// A parsed SemVer always displays canonically and re-parses to an equal value.
    #[test]
    fn semver_display_parse_roundtrip(v in semver_str()) {
        let parsed: SemVer = v.parse().unwrap();
        let shown = parsed.to_string();
        let reparsed: SemVer = shown.parse().unwrap();
        prop_assert_eq!(parsed, reparsed);
        // canonical form is exactly the numeric triple again
        prop_assert_eq!(shown, v);
    }

    /// SemVer display ordering matches numeric ordering component-wise.
    #[test]
    fn semver_ordering_is_numeric(va in semver_str(), vb in semver_str()) {
        let a: SemVer = va.parse().unwrap();
        let b: SemVer = vb.parse().unwrap();
        prop_assert_eq!(
            a < b,
            (a.major(), a.minor(), a.patch()) < (b.major(), b.minor(), b.patch())
        );
    }

    /// A UUID string parsed into a typed ID round-trips through Display + FromStr.
    #[test]
    fn source_id_roundtrip(u in "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}") {
        let id: SourceId = u.parse().unwrap();
        prop_assert_eq!(id.to_string(), u);
        let reparsed: SourceId = id.to_string().parse().unwrap();
        prop_assert_eq!(id, reparsed);
    }

    /// Confidence is closed-additive: any raw float in [0,1] is accepted and preserved.
    #[test]
    fn confidence_accepts_closed_range(v in 0.0f32..=1.0f32) {
        let c = Confidence::new(v).unwrap();
        prop_assert!((c.value() - v).abs() < f32::EPSILON);
    }

    /// A well-formed language tag always round-trips.
    #[test]
    fn language_roundtrip(tag in "([a-z]{2}(-[A-Za-z]{2,8})?)") {
        let lang: Language = tag.parse().unwrap();
        prop_assert_eq!(lang.as_str(), tag);
    }

    /// Timestamp display+parse is identity for values with whole-second precision.
    #[test]
    fn timestamp_display_parse_identity(secs in 0i64..=2_000_000_000i64) {
        let dt = time::OffsetDateTime::from_unix_timestamp(secs).unwrap();
        let ts = Timestamp::from_ymd_hms(
            dt.year(), dt.month() as u8, dt.day(),
            dt.hour(), dt.minute(), dt.second(),
        ).unwrap();
        let reparsed: Timestamp = ts.to_string().parse().unwrap();
        prop_assert_eq!(reparsed, ts);
    }
}
