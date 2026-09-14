//! Stable Quran addressing: the frozen reference grammar (ADR-0102).
//!
//! # quran_core::reference
//!
//! Every stored citation, API reference, and tool result uses this grammar, so
//! parsing and serialization are total, deterministic, and panic-free. The
//! grammar (EBNF):
//!
//! ```ebnf
//! reference     = [ "quran" ":" ] [ edition ":" ] locator ;
//! edition       = slug [ "@" semver ] ;
//! slug          = ALPHA , { ALPHA | DIGIT | "-" } ;
//! locator       = ayah_locator | range_locator | token_locator | division_locator ;
//! ayah_locator  = surah [ ":" ayah ] ;
//! range_locator = surah ":" ayah "-" ( ayah | surah ":" ayah ) ;
//! token_locator = surah ":" ayah ":" ( "token" ":" )? position ;
//! division_locator = ( "juz" | "hizb" | "rub" | "manzil" | "page" | "ruku" | "sajdah" )
//!                    ":" number ;
//! surah = number ; ayah = number ; position = number ;
//! ```
//!
//! Slugs are lowercase. Division keywords take precedence over edition slugs
//! only when the reference is exactly `keyword ":" number`; otherwise a leading
//! keyword-shaped field is an edition slug. Parse requires a locator, so a bare
//! edition is a [`QuranRef::Edition`] only when constructed programmatically.
//!
//! `serialize` emits the deterministic short form that re-parses to the same
//! value (`parse(serialize(r)) == r` for every parseable reference).
//! [`canonical_form`] emits the fully qualified pinned form used for storage and
//! citations, and only exists for pinned ayah-level references.

pub mod parser;
pub mod serializer;

pub use parser::parse;
pub use serializer::{canonical_form, serialize};

use crate::enums::EditionSelector;
use crate::error::QuranError;
use crate::numbers::{AyahNumber, SurahNumber, TokenPosition};
use serde::{Deserialize, Serialize};

/// A structural division addressable by number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivisionKind {
    /// One of 30 parts.
    Juz,
    /// One of 60 parts.
    Hizb,
    /// One of 240 parts.
    Rub,
    /// One of 7 parts.
    Manzil,
    /// A printed page.
    Page,
    /// A bowing section.
    Ruku,
    /// A prostration marker, addressed by occurrence index.
    Sajdah,
}

impl DivisionKind {
    /// The grammar keyword for this division.
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Juz => "juz",
            Self::Hizb => "hizb",
            Self::Rub => "rub",
            Self::Manzil => "manzil",
            Self::Page => "page",
            Self::Ruku => "ruku",
            Self::Sajdah => "sajdah",
        }
    }
}

/// A parsed Quran reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuranRef {
    /// A whole edition (programmatic only; the parser always requires a locator).
    Edition {
        /// The selected edition.
        edition: EditionSelector,
    },
    /// One surah.
    Surah {
        /// The selected edition.
        edition: EditionSelector,
        /// Surah number.
        surah: SurahNumber,
    },
    /// One ayah.
    Ayah {
        /// The selected edition.
        edition: EditionSelector,
        /// Surah number.
        surah: SurahNumber,
        /// Ayah number.
        ayah: AyahNumber,
    },
    /// An inclusive ayah range, possibly spanning surahs.
    AyahRange {
        /// The selected edition.
        edition: EditionSelector,
        /// Range start.
        start: (SurahNumber, AyahNumber),
        /// Range end (`end >= start`).
        end: (SurahNumber, AyahNumber),
    },
    /// One token.
    Token {
        /// The selected edition.
        edition: EditionSelector,
        /// Surah number.
        surah: SurahNumber,
        /// Ayah number.
        ayah: AyahNumber,
        /// Token position.
        position: TokenPosition,
    },
    /// A structural division by number.
    Division {
        /// The selected edition.
        edition: EditionSelector,
        /// Division kind.
        kind: DivisionKind,
        /// Division number (1-based).
        number: u32,
    },
}

impl QuranRef {
    /// The edition selector carried by this reference.
    pub fn edition(&self) -> &EditionSelector {
        match self {
            Self::Edition { edition }
            | Self::Surah { edition, .. }
            | Self::Ayah { edition, .. }
            | Self::AyahRange { edition, .. }
            | Self::Token { edition, .. }
            | Self::Division { edition, .. } => edition,
        }
    }
}

/// A parsed reference plus its canonical serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRef {
    /// The parsed reference.
    pub reference: QuranRef,
    /// `serialize(reference)`.
    pub canonical: String,
}

/// Parse a user-supplied reference and return its canonical serialization.
///
/// This is the parser half of `QuranReader::resolve`; bounds-checking against a
/// loaded edition happens in `quran-corpus` (M8).
pub fn resolve(text: &str) -> Result<ResolvedRef, QuranError> {
    let reference = parse(text)?;
    let canonical = serialize(&reference);
    Ok(ResolvedRef { reference, canonical })
}
