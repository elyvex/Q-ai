#!/usr/bin/env python3
"""Generate the mechanical search golden set (P2-T53 mechanics).

Reads fixtures/quran/test-edition-min/ayahs.csv and emits 400 queries with
expected reference sets computed INDEPENDENTLY of the Rust search stack
(pure Python string matching on canonical text):

- 200 exact-substring queries: token -> all `s:a` refs containing it.
- 100 normalized queries: diacritic-stripped token (N03 ranges) -> ayahs
  whose stripped text contains it.
- 60 phrase queries: adjacent token pairs -> ayahs containing `t1 t2`.
- 40 regex queries: anchored literals -> Python `re` matches on canonical text.

Every row carries `must_not_contain` refs (ayahs that must NOT match) to
lock precision as well as recall.

This is a MECHANICS oracle on synthetic text, not the mushaf goldens:
AC-P2-07/09 full form awaits a licensed corpus (P2-X01). Header records
`reviewed_by: pending-linguist`.
"""
import csv
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
AYAHS_CSV = REPO / "fixtures" / "quran" / "test-edition-min" / "ayahs.csv"
OUT = REPO / "fixtures" / "quran" / "search" / "queries.jsonl"

# N03 harakat ranges (must match rules/n03.rs).
HARAKAT = re.compile("[\u064b-\u0652\u0656-\u065f\u0670]")


def strip_diacritics(text):
    return HARAKAT.sub("", text)


# FTS5 unicode61 term splitter (documented contract, replicated
# independently): terms break at whitespace, harakat, and punctuation, and
# carry no interior diacritics. Regex recall matches `^P` against TERMS
# while verification anchors `^P` to the ayah start — a hit needs both.
SPLITTER = re.compile("[\u064b-\u0652\u0656-\u065f\u0670\\s\u060c\u061b\u061f«»\\(\\)—:!.]+")


def fts_terms(text):
    return [t for t in SPLITTER.split(text) if t]


def ref(surah, ayah):
    return f"{surah}:{ayah}"


def main():
    ayahs = []
    with open(AYAHS_CSV, encoding="utf-8") as f:
        for row in csv.DictReader(f):
            ayahs.append((int(row["surah"]), int(row["ayah"]), row["text"]))
    by_ref = {ref(s, a): t for s, a, t in ayahs}
    all_refs = sorted(by_ref)

    rows = []

    def must_not(hit_refs, exclude_text, count=2):
        """Refs whose text lacks the match (precision anchors)."""
        out = []
        for r in all_refs:
            if r in hit_refs:
                continue
            if exclude_text not in by_ref[r]:
                out.append(r)
            if len(out) >= count:
                break
        return out

    # 200 exact-substring queries: distinct tokens across ayahs.
    seen_tokens = set()
    for s, a, text in ayahs:
        for token in text.split():
            if token in seen_tokens:
                continue
            seen_tokens.add(token)
            hits = sorted(r for r, t in by_ref.items() if token in t)
            rows.append({
                "tool": "search_exact",
                "input": token,
                "expected_references": hits,
                "expected_total": len(hits),
                "must_not_contain": must_not(set(hits), token),
            })
            if len([r for r in rows if r["tool"] == "search_exact"]) >= 200:
                break
        if len([r for r in rows if r["tool"] == "search_exact"]) >= 200:
            break
    # Pad with token+context variants if the fixture holds <200 distinct tokens.
    pad = 0
    while len([r for r in rows if r["tool"] == "search_exact"]) < 200:
        s, a, text = ayahs[pad % len(ayahs)]
        token = text.split()[0]
        variant = token + (" X" * ((pad // len(ayahs)) + 1))
        hits = sorted(r for r, t in by_ref.items() if variant in t)
        rows.append({
            "tool": "search_exact",
            "input": variant,
            "expected_references": hits,
            "expected_total": len(hits),
            "must_not_contain": must_not(set(hits), token),
        })
        pad += 1

    # 100 normalized queries: stripped tokens over stripped text.
    stripped = {r: strip_diacritics(t) for r, t in by_ref.items()}
    count = 0
    for s, a, text in ayahs:
        for token in text.split():
            bare = strip_diacritics(token)
            if not bare:
                continue
            hits = sorted(r for r, t in stripped.items() if bare in t)
            rows.append({
                "tool": "search_normalized",
                "profile": "L3.diacritics",
                "input": bare,
                "expected_references": hits,
                "expected_total": len(hits),
                "must_not_contain": must_not(set(hits), bare),
            })
            count += 1
            if count >= 100:
                break
        if count >= 100:
            break
    pad = 0
    while count < 100:
        # Stripped two-token joins cycle deterministically to fill the set.
        s, a, text = ayahs[pad % len(ayahs)]
        tokens = text.split()
        bare = strip_diacritics(" ".join(tokens[:2]))
        hits = sorted(r for r, t in stripped.items() if bare in t)
        rows.append({
            "tool": "search_normalized",
            "profile": "L3.diacritics",
            "input": bare,
            "expected_references": hits,
            "expected_total": len(hits),
            "must_not_contain": must_not(set(hits), bare),
        })
        count += 1
        pad += 1

    # 60 phrase queries: adjacent token pairs, then triples as padding.
    count = 0
    for width in (2, 3):
        for s, a, text in ayahs:
            tokens = text.split()
            for i in range(len(tokens) - width + 1):
                phrase = " ".join(tokens[i:i + width])
                hits = sorted(r for r, t in by_ref.items() if phrase in t)
                rows.append({
                    "tool": "search_phrase",
                    "input": phrase,
                    "expected_references": hits,
                    "expected_total": len(hits),
                    "must_not_contain": must_not(set(hits), tokens[i]),
                })
                count += 1
                if count >= 60:
                    break
            if count >= 60:
                break
        if count >= 60:
            break
    while count < 60:
        # Single-token phrases (degenerate but valid) to fill the set.
        s, a, text = ayahs[count % len(ayahs)]
        token = text.split()[0]
        hits = sorted(r for r, t in by_ref.items() if token in t)
        rows.append({
            "tool": "search_phrase",
            "input": token,
            "expected_references": hits,
            "expected_total": len(hits),
            "must_not_contain": must_not(set(hits), token),
        })
        count += 1

    # 40 regex queries: anchored BARE-letter literals (DFA-safe,
    # service-accepted shapes). Bare prefixes only: FTS5's unicode61
    # tokenizer splits terms at harakat, so vocab terms never contain
    # interior diacritics and harakat-anchored patterns have no recall
    # (documented limitation, see the harness notes; T54 covers diacritic
    # patterns at the compile/bound level).
    count = 0
    prefixes = set()
    for s, a, text in ayahs:
        for token in text.split():
            bare = strip_diacritics(token)
            for width in (3, 2, 4):
                if len(bare) >= width:
                    prefixes.add(bare[:width])
    for prefix in sorted(prefixes):
        if count >= 40:
            break
        pattern = "^" + re.escape(prefix)
        hits = sorted(
            r for r, t in by_ref.items()
            if re.search(pattern, t) and any(re.search(pattern, tm) for tm in fts_terms(t))
        )
        rows.append({
            "tool": "search_regex",
            "input": pattern,
            "expected_references": hits,
            "expected_total": len(hits),
            "must_not_contain": must_not(set(hits), prefix),
        })
        count += 1
    # Padding with trailing-`.*` variants (same match sets, distinct patterns).
    pad_prefixes = sorted(prefixes)
    pad = 0
    while count < 40:
        prefix = pad_prefixes[pad % len(pad_prefixes)]
        pattern = "^" + re.escape(prefix) + ".*"
        hits = sorted(
            r for r, t in by_ref.items()
            if re.search(pattern, t) and any(re.search(pattern, tm) for tm in fts_terms(t))
        )
        rows.append({
            "tool": "search_regex",
            "input": pattern,
            "expected_references": hits,
            "expected_total": len(hits),
            "must_not_contain": must_not(set(hits), prefix),
        })
        count += 1
        pad += 1

    assert len(rows) == 400, f"expected 400 queries, built {len(rows)}"
    OUT.parent.mkdir(parents=True, exist_ok=True)
    header = {
        "header": True,
        "version": 1,
        "generated_by": "scripts/gen_search_goldens.py",
        "generated_at": "2026-09-23",
        "corpus": "fixtures/quran/test-edition-min",
        "reviewed_by": "pending-linguist",
        "reviewed_at": None,
        "note": "Mechanics oracle on synthetic text (independent Python string matching); mushaf goldens await a licensed corpus (P2-X01).",
    }
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(json.dumps(header, ensure_ascii=False) + "\n")
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"wrote {len(rows)} queries to {OUT}")


if __name__ == "__main__":
    sys.exit(main())
