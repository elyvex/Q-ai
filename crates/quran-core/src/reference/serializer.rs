//! Deterministic serializer for the frozen reference grammar.
//!
//! `serialize` emits the short form that re-parses to the identical value, so
//! `parse(serialize(r)) == r` holds for every parseable reference. The only
//! reference kind the parser cannot produce — a bare [`QuranRef::Edition`] —
//! serializes to its edition segment alone and is excluded from the round-trip
//! property (it is programmatic-only).

use crate::enums::EditionSelector;
use crate::reference::QuranRef;

fn push_edition_prefix(out: &mut String, edition: &EditionSelector) {
    match edition {
        // `Active` and the programmatic-only `Primary` both take the short form
        // (no edition prefix). `Primary` is not part of the frozen reference
        // grammar (ADR-0102) — like a bare `QuranRef::Edition` it is
        // programmatic-only and excluded from the round-trip property; it
        // resolves to a concrete edition before any reference is emitted.
        EditionSelector::Active | EditionSelector::Primary => {}
        EditionSelector::Slug(slug) => {
            out.push_str("quran:");
            out.push_str(slug);
            out.push(':');
        }
        EditionSelector::Pinned { slug, version } => {
            out.push_str("quran:");
            out.push_str(slug);
            out.push('@');
            out.push_str(&version.to_string());
            out.push(':');
        }
    }
}

/// Serialize a reference to its deterministic short form.
pub fn serialize(reference: &QuranRef) -> String {
    let mut out = String::new();
    push_edition_prefix(&mut out, reference.edition());
    match reference {
        QuranRef::Edition { .. } => {}
        QuranRef::Surah { surah, .. } => {
            out.push_str(&surah.to_string());
        }
        QuranRef::Ayah { surah, ayah, .. } => {
            out.push_str(&surah.to_string());
            out.push(':');
            out.push_str(&ayah.to_string());
        }
        QuranRef::AyahRange { start, end, .. } => {
            out.push_str(&start.0.to_string());
            out.push(':');
            out.push_str(&start.1.to_string());
            out.push('-');
            out.push_str(&end.0.to_string());
            out.push(':');
            out.push_str(&end.1.to_string());
        }
        QuranRef::Token { surah, ayah, position, .. } => {
            out.push_str(&surah.to_string());
            out.push(':');
            out.push_str(&ayah.to_string());
            out.push_str(":token:");
            out.push_str(&position.to_string());
        }
        QuranRef::Division { kind, number, .. } => {
            out.push_str(kind.keyword());
            out.push(':');
            out.push_str(&number.to_string());
        }
    }
    out
}

/// The fully qualified pinned form used for storage and citations, e.g.
/// `quran:hafs-uthmani@1.0.0:2:255`.
///
/// Returns `None` unless the reference is ayah-level with a pinned edition.
pub fn canonical_form(reference: &QuranRef) -> Option<String> {
    let (slug, version) = match reference.edition() {
        EditionSelector::Pinned { slug, version } => (slug, version),
        _ => return None,
    };
    let locator = match reference {
        QuranRef::Ayah { surah, ayah, .. } => format!("{surah}:{ayah}"),
        QuranRef::AyahRange { start, end, .. } => {
            format!("{}:{}-{}:{}", start.0, start.1, end.0, end.1)
        }
        QuranRef::Token { surah, ayah, position, .. } => {
            format!("{surah}:{ayah}:token:{position}")
        }
        _ => return None,
    };
    Some(format!("quran:{slug}@{version}:{locator}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::EditionSelector;
    use crate::numbers::{AyahNumber, SurahNumber, TokenPosition};
    use crate::reference::DivisionKind;
    use domain::SemVer;

    fn pinned() -> EditionSelector {
        EditionSelector::Pinned { slug: "hafs-uthmani".into(), version: SemVer::new(1, 0, 0) }
    }

    #[test]
    fn active_refs_serialize_short() {
        let r = QuranRef::Ayah {
            edition: EditionSelector::Active,
            surah: SurahNumber::new(2).unwrap(),
            ayah: AyahNumber::new(255).unwrap(),
        };
        assert_eq!(serialize(&r), "2:255");
        let t = QuranRef::Token {
            edition: EditionSelector::Active,
            surah: SurahNumber::new(2).unwrap(),
            ayah: AyahNumber::new(255).unwrap(),
            position: TokenPosition::new(3).unwrap(),
        };
        assert_eq!(serialize(&t), "2:255:token:3");
        let d = QuranRef::Division {
            edition: EditionSelector::Active,
            kind: DivisionKind::Juz,
            number: 30,
        };
        assert_eq!(serialize(&d), "juz:30");
    }

    #[test]
    fn ranges_serialize_fully_qualified() {
        let r = QuranRef::AyahRange {
            edition: EditionSelector::Slug("hafs-uthmani".into()),
            start: (SurahNumber::new(36).unwrap(), AyahNumber::new(1).unwrap()),
            end: (SurahNumber::new(36).unwrap(), AyahNumber::new(12).unwrap()),
        };
        assert_eq!(serialize(&r), "quran:hafs-uthmani:36:1-36:12");
    }

    #[test]
    fn canonical_form_requires_pinned_ayah_level() {
        let r = QuranRef::Ayah {
            edition: pinned(),
            surah: SurahNumber::new(2).unwrap(),
            ayah: AyahNumber::new(255).unwrap(),
        };
        assert_eq!(canonical_form(&r).as_deref(), Some("quran:hafs-uthmani@1.0.0:2:255"));
        let s = QuranRef::Surah { edition: pinned(), surah: SurahNumber::new(112).unwrap() };
        assert_eq!(canonical_form(&s), None);
        let a = QuranRef::Ayah {
            edition: EditionSelector::Active,
            surah: SurahNumber::new(2).unwrap(),
            ayah: AyahNumber::new(255).unwrap(),
        };
        assert_eq!(canonical_form(&a), None);
    }
}
