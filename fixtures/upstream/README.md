# Vendored upstream mirror — `fawazahmed0/quran-api` (local copy, NOT downloaded)

- **Source repository:** `fawazahmed0/quran-api` (branch `1`)
- **Pinned revision:** `47ca096b0976443ba2eab2e45cdf0fb4096a2610`
- **Copied from:** `/Users/ali/dev/rust/Q-ai-side/references/01/quran-api`
  (local clone, verified byte-identical: `editions.json` sha256
  `062fd8bd…b365`; `diff -r -q` clean on `linebyline/`; spot-`cmp` clean on
  `chapterverse/` including an outlier file)
- **Copied on:** 2026-09-20, by owner direction (offline-first: no network reads)
- **Contents:**
  - `quran-api/editions.json` (236K) — edition/translation catalog metadata
  - `quran-api/database/chapterverse/*.txt` (492 files, 565M) —
    `surah|ayah|text` lines + a 10-line JSON metadata trailer per file
  - `quran-api/database/linebyline/*.txt` (492 files, 548M) —
    bare ordered text lines + the same JSON trailer
- **Corpus facts (binary-measured, not asserted):** every one of the 984 files
  holds exactly 6245 LF bytes = 6236 content lines + the 10-line trailer, with
  no trailing newline; trailer `name` matches the filename slug in all files;
  no CRLF anywhere. Four translation files carry 48 lone-`\r` bytes inside
  verse texts (intra-verse separators); the reader keeps them verbatim, counts
  them (`cr_lines`), and canonical import validation (QV-008) rejects control
  points at promotion time. (An earlier draft of this note blamed Python's
  universal-newline mode for phantom "annotation lines" — corrected 2026-09-20:
  there are no interleaved note lines; `\r` is intra-verse.)
- **Known upstream gaps (fail-closed, per-file):** 3 chapterverse files carry
  empty verse texts the reader rejects — `tam-abdulhameedbaqa-la{,-lad}`
  (`2|282|` empty) and `urd-muhammadtahirul` (`7|1`–`7|4` empty). `qai quran
  catalog --database` reports these as per-file errors without failing the
  run (489/492 chapterverse parse; 492/492 linebyline parse).
- **Licence / redistribution (UNRESOLVED):** the upstream repository is
  Unlicense, but every contained translation/text file keeps its own
  translator/publisher terms, which are **unknown per edition**
  (owner items OD-01 / ODV-02). This mirror is therefore
  **`pending_license_review`**: local development and metadata work only.
  **Pushing or publishing this repository redistributes 492 translations of
  unverified licence status.** Clearing that requires the owner decisions in
  `docs/05-followups/decisions-needed.md` (OD-01) — do not treat this mirror
  as a licence grant.
- **Text-status rule (unchanged):** no vendored file is the canonical corpus.
  Canonical text enters only through `qai quran import` + validation +
  editorial sign-off (ADR-0101, still Draft).
