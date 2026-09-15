//! Phase 2 — bidirectional character-offset maps (`SpanMap`, I10).
//!
//! Every normalization rule emits one [`SpanMap`]; the pipeline composes them so
//! that any match in derived space maps back to exact canonical character
//! ranges (and forward again). Without this, a hit cannot be highlighted in
//! the canonical Uthmani text, cited to a character range, or re-verified by
//! the citation resolver.
//!
//! # Representation (interim, pending ADR-0208)
//!
//! The map stores a total per-character function `derived -> canonical`
//! (every derived char comes from some canonical char; deletions are simply
//! absent from the image). Composition is then plain function composition and
//! is associative by construction. [`SpanMap::segments`] exposes the grouped
//! segment view from the plan sketch for storage and debugging.

use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::error::NormalizationError;
use crate::rule::RuleId;

/// A match location in canonical space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalSpan {
    /// Character range into the canonical text (char indices, end-exclusive).
    pub char_range: Range<u32>,
    /// False when the query range was empty or exceeded the derived text and
    /// had to be clamped; true for a clean in-bounds mapping.
    pub exact: bool,
}

impl CanonicalSpan {
    /// Byte range of [`Self::char_range`] inside `text` (char-indexed).
    ///
    /// Returns `None` when the range exceeds the text. Highlight renderers
    /// slice canonical text with the result; token ranges ride alongside on
    /// the owning hit type.
    #[must_use]
    pub fn byte_range_in(&self, text: &str) -> Option<Range<u32>> {
        let total_chars = text.chars().count() as u32;
        if self.char_range.start > total_chars || self.char_range.end > total_chars {
            return None;
        }
        let byte_of = |idx: u32| {
            if idx == total_chars {
                Some(text.len() as u32)
            } else {
                text.char_indices().nth(idx as usize).map(|(byte, _)| byte as u32)
            }
        };
        Some(byte_of(self.char_range.start)?..byte_of(self.char_range.end)?)
    }
}

/// One grouped segment: a contiguous derived range mapping to a contiguous
/// canonical range, with the rule that produced the mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanSegment {
    /// Range in derived (output) space, end-exclusive.
    pub derived: Range<u32>,
    /// Range in canonical (input) space, end-exclusive.
    pub canonical: Range<u32>,
    /// Rule that produced this segment, if any (`None` = identity carry).
    pub provenance_rule: Option<RuleId>,
}

/// Bidirectional, composable offset map over character indices.
///
/// All offsets are Unicode scalar (`char`) indices, **not** bytes and **not**
/// grapheme clusters. Grapheme-boundary safety is enforced by callers (the
/// pipeline validates ranges against `unicode-segmentation` in M1b); the map
/// itself is a pure index function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanMap {
    /// Length of the input (canonical-side) text in chars.
    canonical_len: u32,
    /// Length of the output (derived-side) text in chars.
    derived_len: u32,
    /// `forward[i]` = canonical index of derived char `i`. Always total:
    /// `forward.len() == derived_len` and every entry is `< canonical_len`
    /// (vacuously true when both are 0).
    forward: Vec<u32>,
    /// Rule that produced each derived char (`None` = identity carry).
    provenance: Vec<Option<RuleId>>,
}

impl SpanMap {
    /// Identity map over `len` chars (profile `L0.exact`).
    #[must_use]
    pub fn identity(len: u32) -> Self {
        Self {
            canonical_len: len,
            derived_len: len,
            forward: (0..len).collect(),
            provenance: vec![None; len as usize],
        }
    }

    /// Build a map from an explicit derived-to-canonical index function.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::InvalidMapping`] when any entry points
    /// outside the canonical text.
    pub fn build(
        canonical_len: u32,
        derived_to_canonical: Vec<u32>,
        rule: RuleId,
    ) -> Result<Self, NormalizationError> {
        if let Some(&bad) = derived_to_canonical.iter().find(|&&c| c >= canonical_len) {
            return Err(NormalizationError::InvalidMapping {
                detail: format!(
                    "derived char maps to canonical index {bad}, out of 0..{canonical_len}"
                ),
            });
        }
        let derived_len = derived_to_canonical.len() as u32;
        Ok(Self {
            canonical_len,
            derived_len,
            forward: derived_to_canonical,
            provenance: vec![Some(rule); derived_len as usize],
        })
    }

    /// Length of the input side in chars.
    #[must_use]
    pub fn canonical_len(&self) -> u32 {
        self.canonical_len
    }

    /// Length of the output side in chars.
    #[must_use]
    pub fn derived_len(&self) -> u32 {
        self.derived_len
    }

    /// Map a match in derived space back to canonical space (I10).
    ///
    /// The range is clamped to the derived text; clamping (or an empty query)
    /// yields `exact = false`. The canonical range is the tight hull over the
    /// mapped chars, so re-slicing canonical text and re-normalizing with the
    /// same profile always contains the matched derived substring.
    #[must_use]
    pub fn to_canonical(&self, derived: Range<u32>) -> CanonicalSpan {
        let start = derived.start.min(self.derived_len);
        let end = derived.end.min(self.derived_len);
        let exact = !derived.is_empty()
            && derived.start <= self.derived_len
            && derived.end <= self.derived_len;
        if start >= end {
            return CanonicalSpan { char_range: start..start, exact: false };
        }
        let slice = &self.forward[start as usize..end as usize];
        let lo = slice.iter().min().copied().unwrap_or(start);
        let hi = slice.iter().max().copied().unwrap_or(start);
        CanonicalSpan { char_range: lo..hi.saturating_add(1), exact }
    }

    /// Map a canonical range forward to derived space.
    ///
    /// Returns `None` when no derived char descends from the range (a fully
    /// deleted region) or when the query range is empty.
    #[must_use]
    pub fn to_derived(&self, canonical: Range<u32>) -> Option<Range<u32>> {
        if canonical.is_empty() {
            return None;
        }
        let start = canonical.start.min(self.canonical_len);
        let end = canonical.end.min(self.canonical_len);
        if start >= end {
            return None;
        }
        let mut iter = self
            .forward
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c >= start && c < end)
            .map(|(i, _)| i as u32);
        let lo = iter.next()?;
        let hi = iter.max().unwrap_or(lo);
        Some(lo..hi.saturating_add(1))
    }

    /// Compose two maps: `self` applied first, then `next`.
    ///
    /// If `self: S0 -> S1` and `next: S1 -> S2`, the result maps `S0 -> S2`
    /// with `result[i] = self[next[i]]`. Panics in debug builds when `next`'s
    /// canonical side disagrees with `self`'s derived side; release builds
    /// clamp defensively.
    #[must_use]
    pub fn compose(&self, next: &SpanMap) -> SpanMap {
        debug_assert_eq!(
            self.derived_len, next.canonical_len,
            "SpanMap::compose requires self.derived_len == next.canonical_len"
        );
        let mut forward = Vec::with_capacity(next.forward.len());
        let mut provenance = Vec::with_capacity(next.forward.len());
        for (i, &mid) in next.forward.iter().enumerate() {
            let mapped = self
                .forward
                .get(mid as usize)
                .copied()
                .unwrap_or_else(|| self.canonical_len.saturating_sub(1).min(mid));
            forward.push(mapped);
            let rule = next
                .provenance
                .get(i)
                .copied()
                .flatten()
                .or_else(|| self.provenance.get(mid as usize).copied().flatten());
            provenance.push(rule);
        }
        SpanMap {
            canonical_len: self.canonical_len,
            derived_len: next.derived_len,
            forward,
            provenance,
        }
    }

    /// Grouped segment view: maximal runs of consecutive derived chars whose
    /// canonical images are also consecutive (and share provenance).
    #[must_use]
    pub fn segments(&self) -> Vec<SpanSegment> {
        let mut out = Vec::new();
        let mut run_start: Option<u32> = None;
        let mut run_canon: u32 = 0;
        let mut run_rule: Option<RuleId> = None;

        let flush = |out: &mut Vec<SpanSegment>,
                     run_start: &mut Option<u32>,
                     run_canon: u32,
                     run_rule: Option<RuleId>,
                     end: u32| {
            if let Some(s) = run_start.take() {
                let len = end - s;
                out.push(SpanSegment {
                    derived: s..end,
                    canonical: run_canon..run_canon.saturating_add(len),
                    provenance_rule: run_rule,
                });
            }
        };

        for (i, &c) in self.forward.iter().enumerate() {
            let i = i as u32;
            let rule = self.provenance[i as usize];
            let continues = match run_start {
                Some(s) => c == run_canon.saturating_add(i - s) && rule == run_rule,
                None => false,
            };
            if continues {
                continue;
            }
            flush(&mut out, &mut run_start, run_canon, run_rule, i);
            run_start = Some(i);
            run_canon = c;
            run_rule = rule;
        }
        let end = self.derived_len;
        flush(&mut out, &mut run_start, run_canon, run_rule, end);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;
    use crate::rule::RuleId;

    fn delete_middle() -> SpanMap {
        // "abc" -> "ac": derived 0 -> canonical 0, derived 1 -> canonical 2.
        SpanMap::build(3, vec![0, 2], RuleId::N02).unwrap()
    }

    #[test]
    fn identity_is_fixed_point_both_directions() {
        let m = SpanMap::identity(4);
        let span = m.to_canonical(1..3);
        assert_eq!(span.char_range, 1..3);
        assert!(span.exact);
        assert_eq!(m.to_derived(1..3), Some(1..3));
    }

    #[test]
    fn deletion_maps_back_to_hull_and_forward_skips_gap() {
        let m = delete_middle();
        let span = m.to_canonical(0..2);
        assert_eq!(span.char_range, 0..3);
        assert!(span.exact);
        // The deleted canonical char still resolves: hull over the survivors.
        assert_eq!(m.to_derived(1..2), None);
        assert_eq!(m.to_derived(0..3), Some(0..2));
    }

    #[test]
    fn out_of_range_clamps_and_marks_inexact() {
        let m = SpanMap::identity(2);
        let span = m.to_canonical(1..9);
        assert_eq!(span.char_range, 1..2);
        assert!(!span.exact);
        assert_eq!(m.to_derived(5..9), None);
    }

    #[test]
    fn empty_identity_round_trips() {
        let m = SpanMap::identity(0);
        assert_eq!(m.derived_len(), 0);
        let span = m.to_canonical(0..0);
        assert!(!span.exact);
    }

    #[test]
    fn build_rejects_dangling_images() {
        let err = SpanMap::build(2, vec![0, 7], RuleId::N03).unwrap_err();
        assert_eq!(err.code(), crate::error::codes::INVALID_MAPPING);
    }

    #[test]
    fn compose_chains_deletions() {
        // S0 "abcd" --rule1--> S1 "acd" --rule2--> S2 "ad".
        let m1 = SpanMap::build(4, vec![0, 2, 3], RuleId::N02).unwrap();
        let m2 = SpanMap::build(3, vec![0, 2], RuleId::N03).unwrap();
        let m = m1.compose(&m2);
        assert_eq!(m.canonical_len(), 4);
        assert_eq!(m.derived_len(), 2);
        let span = m.to_canonical(0..2);
        assert_eq!(span.char_range, 0..4);
        // Direct two-step mapping agrees with the composed map.
        let stepwise = m2.to_canonical(0..2);
        let back = m1.to_canonical(stepwise.char_range);
        assert_eq!(back.char_range, span.char_range);
    }

    #[test]
    fn byte_range_in_maps_multibyte_text() {
        // "بِسْمِ" is 6 chars (letters and harakat are 2 bytes each in UTF-8).
        let text = "بِسْمِ";
        assert_eq!(text.chars().count(), 6);
        let span = CanonicalSpan { char_range: 0..6, exact: true };
        assert_eq!(span.byte_range_in(text), Some(0..text.len() as u32));
        let span = CanonicalSpan { char_range: 2..3, exact: true };
        let bytes = span.byte_range_in(text).unwrap();
        assert_eq!(&text.as_bytes()[bytes.start as usize..bytes.end as usize], "س".as_bytes());
        // Empty range at a boundary maps to the empty byte range there.
        let span = CanonicalSpan { char_range: 6..6, exact: true };
        assert_eq!(span.byte_range_in(text), Some(text.len() as u32..text.len() as u32));
        // Out-of-range maps to nothing.
        let span = CanonicalSpan { char_range: 0..7, exact: false };
        assert_eq!(span.byte_range_in(text), None);
        let span = CanonicalSpan { char_range: 0..0, exact: false };
        assert_eq!(span.byte_range_in(""), Some(0..0));
    }

    #[test]
    fn segments_group_runs_and_mark_provenance() {
        let m = delete_middle();
        let segs = m.segments();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].derived, 0..1);
        assert_eq!(segs[0].canonical, 0..1);
        assert_eq!(segs[0].provenance_rule, Some(RuleId::N02));
        assert_eq!(segs[1].derived, 1..2);
        assert_eq!(segs[1].canonical, 2..3);
        let id = SpanMap::identity(3);
        let segs = id.segments();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].provenance_rule, None);
    }

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        /// Random **valid** chained maps A --a--> B --b--> C --c--> D:
        /// every forward entry points inside its canonical side, so the only
        /// thing under test is composition itself (pure function composition,
        /// associative by construction). Degenerate empty-intermediate chains
        /// are excluded: they are rejected by `SpanMap::build` in real code.
        fn arb_chain() -> impl Strategy<Value = (SpanMap, SpanMap, SpanMap)> {
            (1u32..8, 1u32..8, 1u32..8, 1u32..8).prop_flat_map(|(n0, n1, n2, n3)| {
                (
                    proptest::collection::vec(0..n0, n1 as usize),
                    proptest::collection::vec(0..n1, n2 as usize),
                    proptest::collection::vec(0..n2, n3 as usize),
                )
                    .prop_map(move |(a_fwd, b_fwd, c_fwd)| {
                        let mk = |canon: u32, fwd: Vec<u32>| SpanMap {
                            canonical_len: canon,
                            derived_len: fwd.len() as u32,
                            forward: fwd.clone(),
                            provenance: vec![None; fwd.len()],
                        };
                        (mk(n0, a_fwd), mk(n1, b_fwd), mk(n2, c_fwd))
                    })
            })
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(256))]

            #[test]
            fn composition_is_associative((a, b, c) in arb_chain()) {
                prop_assert_eq!(
                    a.clone().compose(&b).compose(&c),
                    a.compose(&b.compose(&c))
                );
            }

            #[test]
            fn round_trip_containment(len in 0u32..24, del in proptest::collection::vec(proptest::bool::ANY, 0..24)) {
                // Identity with some canonical chars deleted.
                let canon_len = len.max(del.len() as u32).max(1);
                let mut fwd = Vec::new();
                for (i, &drop_it) in del.iter().enumerate() {
                    if !drop_it && (i as u32) < canon_len {
                        fwd.push(i as u32);
                    }
                }
                let m = SpanMap::build(canon_len, fwd, RuleId::N02).unwrap();
                // Every surviving canonical char maps forward and back over itself.
                for c in 0..canon_len {
                    if let Some(d) = m.to_derived(c..c.saturating_add(1)) {
                        let back = m.to_canonical(d.clone());
                        prop_assert!(back.char_range.start <= c && c < back.char_range.end,
                            "c={c} d={d:?} back={back:?}");
                    }
                }
            }
        }
    }
}
