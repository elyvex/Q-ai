//! N16 `nfc` — canonical composition for query text.
//!
//! Applies Unicode NFC (canonical decomposition, canonical ordering, primary
//! composition incl. Hangul) so queries pasted in NFD match NFD-tolerant
//! indexes. On **canonical** text NFC is asserted by Phase 1, never silently
//! applied; this rule exists for the query/index path only.
//!
//! The [`SpanMap`](crate::span::SpanMap) maps each composed char to the source
//! index of its starter (combining marks merge into their starter). NFC is
//! idempotent. Total over all Unicode input (never panics).

use unicode_normalization::char::{canonical_combining_class, compose, decompose_canonical};

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::span::SpanMap;

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NfcCompose;

impl NormalizationRule for NfcCompose {
    fn id(&self) -> RuleId {
        RuleId::N16
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Apply Unicode NFC canonical composition"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        let text = input.text();
        let canonical_len = text.chars().count() as u32;

        // 1. Canonical decomposition, tracking each char's source index.
        let mut decomp: Vec<(char, u32)> = Vec::new();
        for (i, ch) in text.chars().enumerate() {
            decompose_canonical(ch, |d| decomp.push((d, i as u32)));
        }

        // 2. Canonical ordering: stable-sort each starter-led cluster by CCC.
        //    A cluster starts at a CCC-0 char and extends over following
        //    CCC-nonzero chars.
        let mut start = 0;
        while start < decomp.len() {
            let mut end = start + 1;
            while end < decomp.len() && canonical_combining_class(decomp[end].0) != 0 {
                end += 1;
            }
            decomp[start..end].sort_by_key(|(c, _)| canonical_combining_class(*c));
            start = end;
        }

        // 3. Primary composition with blocking: a char composes with the
        //    current starter unless a blocker sits between them (any char
        //    with CCC >= the incoming CCC). For CCC-0 chars every combining
        //    mark is a blocker, so starters only compose when adjacent —
        //    which is exactly the Hangul Jamo case (all Jamo are CCC 0).
        let mut out: Vec<(char, u32)> = Vec::with_capacity(decomp.len());
        let mut starter: Option<usize> = None;
        for (d, src) in decomp {
            let ccc = canonical_combining_class(d);
            let mut consumed = false;
            if let Some(sp) = starter.filter(|sp| *sp < out.len()) {
                let blocked =
                    out[sp + 1..].iter().any(|(c, _)| canonical_combining_class(*c) >= ccc);
                if !blocked && let Some(p) = compose(out[sp].0, d) {
                    out[sp].0 = p;
                    consumed = true;
                }
            }
            if !consumed {
                if ccc == 0 {
                    starter = Some(out.len());
                }
                out.push((d, src));
            }
        }

        let mut composed_text = String::new();
        let mut forward: Vec<u32> = Vec::with_capacity(out.len());
        for (ch, src) in out {
            composed_text.push(ch);
            forward.push(src);
        }
        let rule_map = SpanMap::build(canonical_len, forward, self.id())
            .expect("N16 builds a total map by construction");
        let spans = input.spans().clone().compose(&rule_map);
        NormalizedText::from_parts(composed_text, spans)
    }

    fn is_idempotent(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(input: &str) -> NormalizedText {
        NfcCompose.apply(&NormalizedText::from_plain(input))
    }

    #[test]
    fn composes_decomposed_sequences() {
        // e + combining acute -> precomposed é.
        assert_eq!(apply("e\u{0301}").text(), "\u{00E9}");
        // Alef + maddah above -> precomposed alef-madda.
        assert_eq!(apply("\u{0627}\u{0653}").text(), "\u{0622}");
        // Already-composed Arabic is untouched:
        assert_eq!(apply("بِسْمِ").text(), "بِسْمِ");
        assert_eq!(apply("").text(), "");
    }

    #[test]
    fn canonical_ordering_before_composition() {
        // U+0301 (CCC 230) before U+0327 (CCC 202) reorders to U+0327,U+0301;
        // the acute is not blocked by the lower-class cedilla, so it composes
        // with the starter: NFC("a◌́◌̧") = "á◌̧".
        let out = apply("a\u{0301}\u{0327}");
        assert_eq!(out.text(), "\u{00E1}\u{0327}");
    }

    #[test]
    fn composed_char_maps_to_starter() {
        let out = apply("x\u{0627}\u{0653}y");
        assert_eq!(out.text(), "x\u{0622}y");
        // Derived 1 (composed) maps to canonical 1 (the starter alef).
        let span = out.spans().to_canonical(1..2);
        assert_eq!(span.char_range, 1..2);
        assert!(span.exact);
    }

    #[test]
    fn idempotent_on_mixed_unicode() {
        for s in ["", "abc", "e\u{0301}\u{0327}", "بِسْمِ ٱللَّهِ", "가나", "a\u{034F}b"]
        {
            let once = apply(s);
            let twice = NfcCompose.apply(&once);
            assert_eq!(once.text(), twice.text(), "not idempotent on {s:?}");
        }
    }

    #[test]
    fn matches_reference_nfc() {
        // Cross-check against the crate's own streaming NFC on samples.
        use unicode_normalization::UnicodeNormalization;
        for s in ["", "e\u{0301}", "\u{0627}\u{0653}", "a\u{0301}\u{0327}b", "بِسْمِ", "한국어"]
        {
            let expected: String = s.chars().nfc().collect();
            assert_eq!(apply(s).text(), expected, "NFC mismatch on {s:?}");
        }
    }
}
