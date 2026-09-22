# Bundled-Artifact License Captures

> Standard: A3 (`docs/05-followups/owner-decisions.md` § Addendum 2026-09-22).
> Applies to every bundled artifact: Quran editions (OD-01), debug-reader fonts
> (OD-04), morphology datasets (OD-11), normalization sources (OD-12).

## Rule

No artifact is bundled until its license capture lands in this directory.
Repository open source ≠ contained data redistributable. Until captured, the
artifact stays `unknown` / `pending_owner_verification` and ships via the
user-supplied import path only (OD-01-B pattern).

## Capture template

Each capture is a directory `licenses/<artifact>/` containing:

| File | Contents |
|---|---|
| `LICENSE.txt` | Verbatim raw license text as published by the rights holder |
| `capture.json` | Machine-readable capture record (schema below) |
| `attribution.txt` | Exact attribution string shown to users (if required) |

`capture.json` schema:

```json
{
  "artifact": "tanzil-uthmani-full-1.1",
  "source_url": "https://example.invalid/license",
  "capture_date": "2026-09-22",
  "capturer": "<name of the human who captured it>",
  "spdx_id": "CC-BY-3.0",
  "license_sha256": "sha256:<hex of LICENSE.txt>",
  "redistribution_allowed": true,
  "modification_allowed": false,
  "attribution_required": true,
  "notes": "why this capture authorizes bundling"
}
```

- `source_url`, `capture_date`, `capturer` are **mandatory**. A capture without
  all three is not evidence.
- `spdx_id` is the SPDX identifier **if one exists**; otherwise the source
  wording quoted in `notes`.
- `license_sha256` is the checksum of the raw `LICENSE.txt` in this directory,
  so later edits to the license text are detectable.
- `redistribution_allowed: true` with all fields present is the bundling gate.
  Anything else keeps the artifact on the user-import path.

## Current captures

| Artifact | Status |
|---|---|
| `tanzil/` (Tanzil full Uthmani) | 🔴 pending — OD-01 bundling gate |
| `amiri/` (Amiri Quran font, Phase-2 candidate) | 🔴 pending — OD-04 |
| `qac/` (Quranic Arabic Corpus morphology) | 🔴 pending — OD-11 |

Nothing here reopens a 🔴 item: the six pending human inputs
(`owner-decisions.md`) stand. This directory only tightens what happens once
each lands.
