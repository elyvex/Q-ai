#!/usr/bin/env python3
"""Generate the concatenated (spaceless) search golden set (SC2, G-03).

Companion to ``scripts/gen_search_goldens.py``. Reads
``fixtures/quran/test-edition-min/ayahs.csv`` and emits
``fixtures/quran/search/concatenated.jsonl`` with ``search_concatenated`` rows
whose expected reference sets and segmentations are computed INDEPENDENTLY of
the Rust search stack (pure Python).

The oracle applies **L6.skeleton's full ordered rule list** exactly as the
runner does (``crates/quran-normalization/src/profile.rs`` builtin ladder):

    N01,N11,N16,N04,N14,N03,N05,N02,N07,N06,N08,N10,N13,N09,N15,N12,N17

and models the same skeleton store the service searches:

- one L6 skeleton per ayah;
- one L6 skeleton per sliding 3-ayah window (stride 1, surah-scoped),
  mirroring ``skeletons_for_surah`` (``crates/quran-search/src/skeleton.rs``)
  and migration 0014's ``ayah_end - ayah_start <= 2`` CHECK. The builder
  normalizes the raw texts joined with single spaces; N17 then deletes that
  space, so ``L6(join(texts, " "))`` equals the concatenation of the three
  per-ayah L6 skeletons. This oracle asserts that identity and uses the
  per-ayah concatenation to map a window match's character offset back to the
  overlapped ayah numbers.

This is a MECHANICS oracle on synthetic text, not the mushaf goldens
(AC-P2-08 full form awaits a licensed corpus). The header records
``reviewed_by: pending-linguist`` (OD-12).
"""
import csv
import json
import sys
import unicodedata
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
AYAHS_CSV = REPO / "fixtures" / "quran" / "test-edition-min" / "ayahs.csv"
OUT = REPO / "fixtures" / "quran" / "search" / "concatenated.jsonl"

# ── L6 rule char sets (must match crates/quran-normalization/src/rules/*.rs) ──

# N11 invisible format controls.
_INVISIBLE = (
    set(chr(c) for c in range(0x200B, 0x2010))
    | set(chr(c) for c in range(0x202A, 0x202F))
    | set(chr(c) for c in range(0x2066, 0x206A))
    | {"\uFEFF"}
)

# N04 Uthmani annotation marks.
_QURANIC_MARKS = (
    set(chr(c) for c in range(0x06D6, 0x06EE))
    | {"\u0615"}
    | set(chr(c) for c in range(0x0617, 0x061B))
)

# N14 waqf pause letters (subset of N04).
_PAUSE_MARKS = set(chr(c) for c in range(0x06D6, 0x06DC))

# N03 harakat.
_HARAKAT = (
    set(chr(c) for c in range(0x064B, 0x0653))
    | set(chr(c) for c in range(0x0656, 0x065A))
    | set(chr(c) for c in range(0x065A, 0x0660))
)

# N12 Arabic + Latin punctuation (letters/digits excluded).
_PUNCT = (
    set(chr(c) for c in range(0x0021, 0x0030))
    | set(chr(c) for c in range(0x003A, 0x0041))
    | set(chr(c) for c in range(0x005B, 0x0061))
    | set(chr(c) for c in range(0x007B, 0x007F))
    | {"\u060C", "\u061B", "\u061F", "\u066A", "\u066B", "\u066C", "\u066D", "\u06D4"}
)

# N06 hamza carriers (standalone hamza deleted).
_N06 = {"\u0622": "\u0627", "\u0623": "\u0627", "\u0625": "\u0627",
        "\u0624": "\u0648", "\u0626": "\u064A", "\u0621": ""}

# N10 Persian-keyboard folds.
_N10 = {"\u06A9": "\u0643", "\u06CC": "\u064A", "\u06C0": "\u0647", "\u06C1": "\u0647"}

# N15 lam-alef presentation ligatures → base sequences.
_N15 = {
    "\uFEF5": "\u0644\u0622", "\uFEF6": "\u0644\u0622",
    "\uFEF7": "\u0644\u0623", "\uFEF8": "\u0644\u0623",
    "\uFEF9": "\u0644\u0625", "\uFEFA": "\u0644\u0625",
    "\uFEFB": "\u0644\u0627", "\uFEFC": "\u0644\u0627",
}


def _map_chars(text, table):
    return "".join(table.get(ch, ch) for ch in text)


def _fold_digits(text):
    out = []
    for ch in text:
        code = ord(ch)
        if 0x0660 <= code <= 0x0669:
            out.append(chr(code - 0x0660 + ord("0")))
        elif 0x06F0 <= code <= 0x06F9:
            out.append(chr(code - 0x06F0 + ord("0")))
        else:
            out.append(ch)
    return "".join(out)


def l6(text):
    """Apply L6.skeleton's full ordered rule list to ``text``."""
    text = " ".join(text.split())                       # N01 whitespace collapse
    text = "".join(ch for ch in text if ch not in _INVISIBLE)   # N11
    text = unicodedata.normalize("NFC", text)           # N16
    text = "".join(ch for ch in text if ch not in _QURANIC_MARKS)  # N04
    text = "".join(ch for ch in text if ch not in _PAUSE_MARKS)    # N14
    text = "".join(ch for ch in text if ch not in _HARAKAT)        # N03
    text = text.replace("\u0670", "")                   # N05
    text = text.replace("\u0640", "")                   # N02
    text = text.replace("\u0671", "\u0627")             # N07
    text = _map_chars(text, _N06)                       # N06
    text = text.replace("\u0649", "\u064A")             # N08
    text = _map_chars(text, _N10)                       # N10
    text = _fold_digits(text)                           # N13
    text = text.replace("\u0629", "\u0647")             # N09
    text = _map_chars(text, _N15)                       # N15
    text = "".join(ch for ch in text if ch not in _PUNCT)  # N12
    text = text.replace(" ", "")                        # N17
    return text


def ref(surah, ayah):
    return f"{surah}:{ayah}"


def split_ref(r):
    surah, ayah = r.split(":")
    return int(surah), int(ayah)


def sort_key(r):
    return split_ref(r)


def main():
    ayahs = []
    with open(AYAHS_CSV, encoding="utf-8") as f:
        for row in csv.DictReader(f):
            ayahs.append((int(row["surah"]), int(row["ayah"]), row["text"]))
    ayahs.sort(key=lambda t: (t[0], t[1]))

    # ── Ayah skeletons + per-token skeletons (tiling identity) ────────────────
    ayah_skel = {}
    ayah_tokens = {}  # ref -> [(position, surface, derived_skeleton), ...]
    for surah, ayah, text in ayahs:
        r = ref(surah, ayah)
        ayah_skel[r] = l6(text)
        tokens = []
        for position, surface in enumerate(text.split(), start=1):
            tokens.append((position, surface, l6(surface)))
        ayah_tokens[r] = tokens
        joined = "".join(t[2] for t in tokens)
        assert joined == ayah_skel[r], f"token tiling broken for {r}: {joined!r} != {ayah_skel[r]!r}"

    all_refs = sorted(ayah_skel, key=sort_key)

    # ── Window skeletons (sliding 3-ayah, stride 1, surah-scoped) ─────────────
    windows = []  # (surah, [ayah_numbers], per_ayah_skeletons, window_skeleton)
    by_surah = {}
    for surah, ayah, _ in ayahs:
        by_surah.setdefault(surah, []).append(ayah)
    for surah, numbers in by_surah.items():
        exactly = [a for a in numbers if (surah, a) in {(s, a) for s, a, _ in ayahs}]
        # Only contiguous runs support windows; the fixture is contiguous.
        for i in range(len(exactly) - 2):
            three = exactly[i:i + 3]
            if three[1] != three[0] + 1 or three[2] != three[1] + 1:
                continue
            per = [ayah_skel[ref(surah, a)] for a in three]
            joined_texts = " ".join(
                next(t for s, a, t in ayahs if s == surah and a == num) for num in three
            )
            window_skel = l6(joined_texts)
            # Builder identity: L6(join) == concat of per-ayah L6 skeletons.
            assert window_skel == "".join(per), (
                f"window identity broken surah {surah} {three}: "
                f"{window_skel!r} != {''.join(per)!r}"
            )
            windows.append((surah, three, per, window_skel))

    # ── Reference-set model ───────────────────────────────────────────────────
    def ayah_matches(query):
        return sorted((r for r in all_refs if query in ayah_skel[r]), key=sort_key)

    def window_bounds(per):
        cum, start = [], 0
        for sk in per:
            cum.append((start, start + len(sk)))
            start += len(sk)
        return cum

    def matching_windows(query):
        """Windows whose skeleton contains `query`, with the FIRST-occurrence span."""
        out = []
        for surah, three, per, skel in windows:
            idx = skel.find(query)
            if idx < 0:
                continue
            end = idx + len(query)
            cum = window_bounds(per)
            overlapped = [a for a, (lo, hi) in zip(three, cum) if lo < end and idx < hi]
            out.append((surah, three, per, cum, idx, end, overlapped))
        return out

    def window_matches(query):
        """(window_refs, boundary_refs) using each window's FIRST occurrence."""
        wrefs, brefs = set(), set()
        for surah, _three, _per, _cum, _idx, _end, overlapped in matching_windows(query):
            for a in overlapped:
                wrefs.add(ref(surah, a))
            if len(overlapped) > 1:
                for a in overlapped:
                    r = ref(surah, a)
                    if query not in ayah_skel.get(r, ""):
                        brefs.add(r)
        return wrefs, brefs

    def expected_refs(query, allow_cross_ayah):
        refs = set(ayah_matches(query))
        brefs = set()
        if allow_cross_ayah:
            wrefs, brefs = window_matches(query)
            refs |= wrefs
        return refs, brefs

    def segmentation_of(r, query):
        """Model `segment_concatenated` for the first occurrence in ayah `r`."""
        surah, ayah = split_ref(r)
        skel = ayah_skel[r]
        idx = skel.find(query)
        assert idx >= 0, f"{query!r} not in {r} skeleton"
        match_start, match_end = idx, idx + len(query)
        parts = []
        offset = 0
        for position, surface, tok_skel in ayah_tokens[r]:
            tok_start, tok_end = offset, offset + len(tok_skel)
            offset = tok_end
            if tok_end <= match_start or tok_start >= match_end:
                continue
            lo = max(match_start, tok_start) - match_start
            hi = min(match_end, tok_end) - match_start
            part = query[lo:hi]
            if part:
                parts.append({
                    "query_part": part,
                    "canonical_token": position,
                    "surah": surah,
                    "ayah": ayah,
                    "position": len(parts),
                })
        assert parts, f"empty segmentation for {query!r} on {r}"
        assert "".join(p["query_part"] for p in parts) == query, (
            f"segmentation does not tile {query!r} on {r}"
        )
        return parts

    def must_not(refs, query, count=2):
        out = []
        for r in all_refs:
            if r in refs or query in ayah_skel[r]:
                continue
            out.append(r)
            if len(out) >= count:
                break
        return out

    def make_row(query, allow_cross_ayah, max_ayah_span=3, input_text=None,
                 expected_rules_contain=None, segmentation=None):
        refs, brefs = expected_refs(query, allow_cross_ayah)
        assert refs, f"no references for {query!r} (allow_cross_ayah={allow_cross_ayah})"
        first = sorted(refs, key=sort_key)[0]
        if segmentation is None:
            segmentation = segmentation_of(first, query)
        row = {
            "tool": "search_concatenated",
            "input": input_text if input_text is not None else query,
            "allow_cross_ayah": allow_cross_ayah,
            "max_ayah_span": max_ayah_span,
            "expected_references": sorted(refs, key=sort_key),
            "expected_total": len(refs),
            "must_not_contain": must_not(refs, query),
            "expected_segmentation": segmentation,
        }
        if brefs:
            row["expected_boundary_refs"] = sorted(brefs, key=sort_key)
        if expected_rules_contain:
            row["expected_rules_contain"] = expected_rules_contain
        return row

    def window_segmentation(query):
        """Model the service's cross-ayah window tiling for the unique matching
        window: per overlapped ayah, the query slice explained by each canonical
        token, in canonical (surah, ayah, position) order."""
        matches = matching_windows(query)
        assert len(matches) == 1, f"expected exactly one matching window for {query!r}"
        surah, three, per, cum, idx, end, _overlapped = matches[0]
        parts = []
        for ayah, (ayah_lo, _ayah_hi) in zip(three, cum):
            offset = 0
            for position, _surface, tok_skel in ayah_tokens[ref(surah, ayah)]:
                g_lo, g_hi = ayah_lo + offset, ayah_lo + offset + len(tok_skel)
                offset += len(tok_skel)
                if g_hi <= idx or g_lo >= end:
                    continue
                lo = max(idx, g_lo) - idx
                hi = min(end, g_hi) - idx
                part = query[lo:hi]
                if part:
                    parts.append({
                        "query_part": part,
                        "canonical_token": position,
                        "surah": surah,
                        "ayah": ayah,
                        "position": len(parts),
                    })
        assert parts, f"empty window segmentation for {query!r}"
        assert "".join(p["query_part"] for p in parts) == query, (
            f"window segmentation does not tile {query!r}"
        )
        return parts

    rows = []
    seen_inputs = set()

    def add(row):
        if row["input"] in seen_inputs:
            return False
        seen_inputs.add(row["input"])
        rows.append(row)
        return True

    # ── A. Corpus-derivable anchor: `سانبل` → 3:3, two-part segmentation ──────
    anchor_query = l6("\u0633\u0627") + l6("\u0646\u0628\u0644")  # tokens 3+4 of 3:3
    anchor = make_row(anchor_query, allow_cross_ayah=False)
    assert anchor["expected_references"] == ["3:3"], anchor["expected_references"]
    assert len(anchor["expected_segmentation"]) == 2, anchor["expected_segmentation"]
    assert [p["canonical_token"] for p in anchor["expected_segmentation"]] == [3, 4]
    add(anchor)

    # ── B. Ayah-level rows: token joins + token triples + bare tokens ─────────
    for surah, ayah, _ in ayahs:
        r = ref(surah, ayah)
        toks = ayah_tokens[r]
        defs = [t[2] for t in toks]
        candidates = []
        for width in (2, 3):
            for i in range(len(defs) - width + 1):
                candidates.append("".join(defs[i:i + width]))
        for d in defs:
            if len(d) >= 2:
                candidates.append(d)
        for query in candidates:
            if len(query) < 2:
                continue
            if query not in ayah_skel[r]:
                continue
            add(make_row(query, allow_cross_ayah=False))

    # ── C. Cross-ayah rows: window substrings spanning an ayah junction ───────
    for surah, three, per, skel in windows:
        junction1 = len(per[0])
        junction2 = len(per[0]) + len(per[1])
        pair_at = {junction1: (three[0], three[1]), junction2: (three[1], three[2])}
        for junction, (a_left, a_right) in pair_at.items():
            for left_len in (1, 2, 3):
                for right_len in (1, 2, 3):
                    start = junction - left_len
                    end = junction + right_len
                    if start < 0 or end > len(skel):
                        continue
                    query = skel[start:end]
                    if len(query) < 3:
                        continue
                    # Clean cross-ayah case: exactly one window matches, it
                    # overlaps exactly the intended pair, and neither overlapped
                    # ayah is an ayah-level hit — so both boundary parts survive
                    # dedup and tile the whole query.
                    matches = matching_windows(query)
                    if len(matches) != 1:
                        continue
                    _s, _t, _p, _c, _i, _e, overlapped = matches[0]
                    if sorted(overlapped) != sorted([a_left, a_right]):
                        continue
                    ayah_hits = set(ayah_matches(query))
                    if any(ref(surah, a) in ayah_hits for a in overlapped):
                        continue
                    seg = window_segmentation(query)
                    row = make_row(query, allow_cross_ayah=True, segmentation=seg)
                    assert row.get("expected_boundary_refs"), (
                        f"cross-ayah row lost boundary refs: {query!r}"
                    )
                    add(row)

    # ── D. Persian-codepoint rows: ی/ک fold to ي/ك by N10 (disclosed) ─────────
    persian_done = 0
    for surah, ayah, _ in ayahs:
        if persian_done >= 10:
            break
        r = ref(surah, ayah)
        toks = ayah_tokens[r]
        defs = [t[2] for t in toks]
        for width in (2, 1):
            for i in range(len(defs) - width + 1):
                query = "".join(defs[i:i + width])
                if len(query) < 3:
                    continue
                if "\u064A" not in query and "\u0643" not in query:
                    continue
                # Persian-keyboard input: ي→ی, ك→ک. L6 folds both back.
                typed = query.replace("\u064A", "\u06CC").replace("\u0643", "\u06A9")
                assert l6(typed) == query, f"persian fold broken: {typed!r} -> {l6(typed)!r}"
                row = make_row(
                    query,
                    allow_cross_ayah=False,
                    input_text=typed,
                    expected_rules_contain=["N10"],
                )
                if row["expected_references"]:
                    if add(row):
                        persian_done += 1
                break
            if persian_done >= 10:
                break

    # ── E. Ayah-level substrings: interior slices of each ayah skeleton ───────
    for surah, ayah, _ in ayahs:
        r = ref(surah, ayah)
        skel = ayah_skel[r]
        for length in (5, 4, 3, 6):
            for start in range(0, len(skel) - length + 1):
                query = skel[start:start + length]
                add(make_row(query, allow_cross_ayah=False))

    # ── Header + write ────────────────────────────────────────────────────────
    assert len(rows) >= 120, f"expected >= 120 rows, built {len(rows)}"
    cross = [r for r in rows if r["allow_cross_ayah"]]
    persian = [r for r in rows if r.get("expected_rules_contain")]
    assert len(cross) >= 10, f"expected >= 10 cross-ayah rows, built {len(cross)}"
    assert len(persian) >= 10, f"expected >= 10 persian rows, built {len(persian)}"
    for row in cross:
        assert row["expected_references"], f"cross-ayah row with empty refs: {row['input']!r}"
        assert row.get("expected_boundary_refs"), f"cross-ayah row without boundary refs: {row['input']!r}"

    OUT.parent.mkdir(parents=True, exist_ok=True)
    header = {
        "header": True,
        "version": 1,
        "generated_by": "scripts/gen_concatenated_goldens.py",
        "generated_at": "2026-09-28",
        "corpus": "fixtures/quran/test-edition-min",
        "reviewed_by": "pending-linguist",
        "reviewed_at": None,
        "note": (
            "Mechanics oracle on synthetic text (independent Python L6 skeleton + "
            "3-ayah window matching); mushaf goldens await a licensed corpus (P2-X01)."
        ),
    }
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(json.dumps(header, ensure_ascii=False) + "\n")
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"wrote {len(rows)} concatenated queries to {OUT}")
    print(f"  cross-ayah rows: {len(cross)}; persian rows: {len(persian)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
