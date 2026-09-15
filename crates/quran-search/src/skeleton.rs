//! Phase 2 — letter-skeleton builder for concatenated search (D2.4 slice).
//!
//! Skeletons are `L6.skeleton` normalizations: spaceless, punctuation-free
//! letter sequences. Two levels ship here:
//!
//! - **ayah**: one skeleton per ayah (most queries);
//! - **window**: one skeleton per sliding 3-ayah window (stride 1,
//!   surah-scoped) for phrases straddling an ayah boundary.
//!
//! Windows normalize the **joined raw texts**, not concatenated skeletons, so
//! boundary normalization (whitespace collapse, NFC) sees the real context.
//! Matches on windows are labeled `spans_ayah_boundary` downstream (M3) and
//! deduplicated against ayah matches there — never presented as one verse.

use quran_normalization::{NormalizationPipeline, ProfileId, ProfileRegistry, SemVer};

/// One built skeleton with its source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltSkeleton {
    /// 1-based ayah number of the window start.
    pub ayah_start: u32,
    /// 1-based ayah number of the window end (`== ayah_start` for singles).
    pub ayah_end: u32,
    /// The `L6.skeleton` string.
    pub skeleton: String,
}

/// Build the L6 pipeline from the built-in ladder (version-pinned).
fn skeleton_pipeline() -> NormalizationPipeline {
    let registry = ProfileRegistry::new();
    NormalizationPipeline::for_profile(&registry, ProfileId::L6, SemVer::new(1, 0, 0))
        .expect("built-in L6 ladder is always well-formed")
}

/// Skeleton for one ayah text.
#[must_use]
pub fn ayah_skeleton(text: &str) -> BuiltSkeleton {
    // Ayah numbers are attached by the caller (see `skeletons_for_surah`).
    let (derived, _) = skeleton_pipeline().apply(text);
    BuiltSkeleton { ayah_start: 0, ayah_end: 0, skeleton: derived.text().to_string() }
}

/// Skeletons for one surah: every ayah plus every 3-ayah window.
///
/// `ayahs` holds `(ayah_number, text)` in canonical order. Windows join the
/// three raw texts with single spaces before normalizing.
#[must_use]
pub fn skeletons_for_surah(ayahs: &[(u32, String)]) -> Vec<BuiltSkeleton> {
    let pipeline = skeleton_pipeline();
    let mut out = Vec::new();
    for (number, text) in ayahs {
        let (derived, _) = pipeline.apply(text);
        out.push(BuiltSkeleton {
            ayah_start: *number,
            ayah_end: *number,
            skeleton: derived.text().to_string(),
        });
    }
    for window in ayahs.windows(3) {
        let joined = window.iter().map(|(_, text)| text.as_str()).collect::<Vec<_>>().join(" ");
        let (derived, _) = pipeline.apply(&joined);
        out.push(BuiltSkeleton {
            ayah_start: window[0].0,
            ayah_end: window[2].0,
            skeleton: derived.text().to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 4] = ["بِسْمِ", "ٱللَّهِ", "ٱلرَّحْمَٰنِ", "ٱلرَّحِيمِ"];

    #[test]
    fn ayah_skeleton_is_spaceless_bare() {
        let built = ayah_skeleton("بِسْمِ ٱللَّهِ");
        assert_eq!(built.skeleton, "بسمالله");
    }

    #[test]
    fn surah_skeletons_cover_ayahs_and_windows() {
        let ayahs: Vec<(u32, String)> =
            WORDS.iter().enumerate().map(|(i, w)| (i as u32 + 1, (*w).to_string())).collect();
        let built = skeletons_for_surah(&ayahs);
        // 4 singles + 2 windows (1-3, 2-4).
        assert_eq!(built.len(), 6);
        assert_eq!(built[0].ayah_start, 1);
        assert_eq!(built[0].ayah_end, 1);
        assert_eq!(built[4].ayah_start, 1);
        assert_eq!(built[4].ayah_end, 3);
        assert_eq!(built[5].ayah_start, 2);
        assert_eq!(built[5].ayah_end, 4);
        // Every single-ayah skeleton is a substring of a covering window.
        assert!(built[4].skeleton.contains(&built[0].skeleton));
    }

    #[test]
    fn short_surahs_emit_no_windows() {
        let ayahs: Vec<(u32, String)> =
            WORDS[..2].iter().enumerate().map(|(i, w)| (i as u32 + 1, (*w).to_string())).collect();
        let built = skeletons_for_surah(&ayahs);
        assert_eq!(built.len(), 2);
        assert!(built.iter().all(|b| b.ayah_start == b.ayah_end));
    }
}
