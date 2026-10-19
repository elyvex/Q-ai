# Phase 4 — UI Review

**Audited:** 2026-10-13
**Baseline:** 04-UI-SPEC.md (checker-approved 7/7)
**Screenshots:** not captured (no dev server — CLI + static-file surface, no localhost:3000/5173/8080)
**Interaction captures:** off (workflow.ui_interaction_capture is false; findings below are code-derived and say so)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | All contract strings verbatim except `Next:` commands omit the `quran` segment |
| 2. Visuals | 4/4 | Summary-first hierarchy holds on every surface; no icon-only actions |
| 3. Color | 4/4 | Zero CLI color codes; SVG hex exact; `#a00` reserved for truncation banners |
| 4. Typography | 4/4 | 3 sizes / 2 weights as declared; `direction="rtl"` root present |
| 5. Spacing | 4/4 | 0/2/6-space terminal grid verified; px scale reserved-unused as declared |
| 6. Experience Design | 3/4 | 41/41 states covered; `dispute` service op has no CLI verb |

**Overall: 22/24**

---

## Top 3 Priority Fixes

1. **`Next:` commands name a non-existent path (`qai graph …` instead of `qai quran graph …`)** — pasting the suggested command fails; prefix every `next_command()` in `crates/quran-graph/src/error.rs` with the `quran` segment (WARNING)
2. **`dispute` review operation is service-only with no CLI verb** — operators cannot record a dispute from the CLI though the decision value renders in review lines; add a `review dispute` verb or document the gap (WARNING)
3. **SVG has no `<title>` element** — `role="img"` + `<desc>` exist but screen readers prefer `<title>`; one-line addition to `render_svg` (minor)

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

Verified verbatim against the contract:

- Neighbor summary `neighbors of {node}: {} node(s), {} edge(s)` — `crates/application/src/quran_cli.rs:3696,3818` ✓
- Empty states are zero-count lines, never generic strings; path absence renders `no path (complete search)` only when complete — `quran_cli.rs:4158` ✓
- Truncated searches render `unknown (truncated)`, never `no path` — `quran_cli.rs:4126,4159` ✓
- Truncation marker ` (truncated)` + `\nincomplete: {detail}` verbatim — `quran_cli.rs:3557,3560` ✓
- Error form `[QAI-GRAPH-NNNN] <detail>` + `Remedy:` + `Next:` via `render_human` — `crates/quran-graph/src/error.rs:70-82` ✓
- Usage/format strings exact: `specify exactly one of --file or --db`, `` unknown export format `x`; use json, dot, or svg `` — `quran_cli.rs:3573,4385` ✓
- Word-root gate typed `[QAI-MORPH-0004]`, never an empty list ✓
- Quotation fail-closed: `quotation unavailable for '<stable-id>': …` — `quran_cli.rs:3806` ✓
- Review input: `bad evidence JSON: …`, `reviewer and method must both be non-empty (OD-02)` — `quran_cli.rs:651,4667` ✓
- Review line `<verb> <id> (<decision>) by <reviewer> at <ts>[(reused)]` — `quran_cli.rs:4688-4701` ✓; `correct` renders `(supersedes <old>)` — `quran_cli.rs:4870` ✓
- Doctor line `GRAPH DOCTOR` + `[<STATUS>] <id> — <summary>` + 6-space remedy/next — `quran_cli.rs:4894-4907` ✓
- Export line `exported <n> nodes, <m> edges, <k> assertions as Graph JSON (projection <id>)` — `quran_cli.rs:4483,4491` ✓
- Destructive gate exact: `Type APPROVE to <verb> \`<target>\``, `refusing to <verb> without --yes`, `not approved` — `crates/cli/src/quran.rs:1703-1714` ✓

**WARNING — wrong `Next:` command path:** `GraphError::next_command()` emits `qai graph build --help`, `qai graph neighbors --help`, `qai graph subgraph --help`, `qai graph pattern --help`, `qai graph inspect --help`, `qai quran graph review --help` (`error.rs:218-229`). The real tree is `qai quran …` (`crates/cli/src/lib.rs:1318`, `quran.rs:1`) — there is no top-level `qai graph` command, so five of the six suggested commands fail if pasted. (Only the review one carries the `quran` segment.) The UI-SPEC copy row repeats the shortened form, so spec and code agree with each other but disagree with the binary. Fix in `error.rs`; update the spec row to match.

### Pillar 2: Visuals (4/4)

- Focal point per surface holds: every read verb prints the column-0 count line first, detail indented below (`quran_cli.rs:3696-3818`, path/subgraph/pattern sections) ✓
- Review verbs lead with the `<verb> <id> (<decision>)` line; doctor leads with `GRAPH DOCTOR` then one `[STATUS]` line per check ✓
- Export leads with the `exported …` line; DOT lists nodes sorted by stable ID then edges; SVG layers seed-first with the truncation banner last ✓
- No icon-only actions exist (text-only CLI); every status is a word (`yes`, `no path (complete search)`, `unknown (truncated)`, `pending`), never color or glyph alone ✓
- SVG carries `role="img"` + `<desc>quran-graph-json-v1 neighborhood of …</desc>` (`quran_graph_export.rs:236-240`) ✓
- Minor (not deducted): no `<title>` element in the SVG; see fix #3.

### Pillar 3: Color (4/4)

- CLI emits zero color codes — dominant 60% is the operator terminal theme by construction (no ANSI escapes in `quran_cli.rs` graph renderers) ✓
- SVG ink exact: `#eef` fill / `#448` stroke (nodes), `#888` edge lines, `#555` edge labels — `quran_graph_export.rs:248-262` ✓
- Accent `#a00` appears exactly once: the SVG truncation banner (`quran_graph_export.rs:267-273`); DOT uses the monochrome `__truncated` plaintext node + `// truncated:` comment in the same slot (`quran_graph_export.rs:167-205`) ✓
- Destructive repair is gated by APPROVE text, never by color alone ✓

### Pillar 4: Typography (4/4)

- CLI body: terminal default, regular, single-spaced rows ✓
- SVG labels: `font-size="10"` regular (node IDs, edge predicates); banner `font-size="13"` + `font-weight="bold"` — `quran_graph_export.rs:248-273` ✓
- 3 sizes, 2 weights — no undeclared sizes found in graph renderers ✓
- SVG root declares `direction="rtl"` for mixed-direction labels (`quran_graph_export.rs:236-240`) ✓
- Arabic text never typeset by this phase (refs + hashes only) ✓

### Pillar 5: Spacing (4/4)

- Summary lines at column 0; quotation/path-step indent exactly 2 spaces (`  ayah:…`, `  N. …`) ✓
- Doctor remedy/next indent exactly 6 spaces (`      remedy:`, `      next:`) — `quran_cli.rs:4903,4906` ✓
- Single-spaced, one finding per line throughout ✓
- Invariant `(s)` suffix at every count — no singular/plural branching ✓
- No px rendering; declared scale reserved-unused with zero exceptions ✓
- SVG/DOT coordinates presentational only, never asserted (04-03 loaders pin this) ✓

### Pillar 6: Experience Design (3/4)

Code-derived (no interaction capture in this run):

- Loading: none by design — reads and review/doctor verbs run synchronously; correct for a local CLI ✓
- Error: every failure carries code + remedy + next command; unknown node/projection → NOT_FOUND, budget/pattern → VALIDATION, denials → POLICY ✓
- Empty: zero-count summaries with exit 0; unknown subgraph seeds skip to complete-empty; fresh DB doctor reports `Skipped` with the build remedy ✓
- Partial: ` (truncated)` + `incomplete:` on the summary line; truncated paths report `unknown`, never absence; partial exports carry the notice inside the `quran-graph-json-v1` envelope ✓
- Overflow: lists stream to stdout (scrollback is the container); SVG width formula `160px × widest layer + 80`, one row per hop depth ✓
- Destructive: `--yes`/APPROVE gate with service-side refusal; success renders `<summary> (audit #<seq>)` with before-and-after counts ✓
- Review: suggestions stay pending with algorithm attribution until a recorded human decision; tombstoned rows hidden immediately, retained for audit ✓

**WARNING — `dispute` has no CLI verb:** the `Disputed` decision renders in review lines and stays effective in traversal, but operators can only reach it via the service API — `accept/reject/correct` cover the CLI while `dispute` (wired at the service layer since 04-02, carried through 04-05 summaries) has no operator surface. Either add `review dispute` or record the deferral explicitly.

---

## Files Audited

- `crates/application/src/quran_cli.rs` (neighbor/path/subgraph/pattern/export/review/doctor renderers, truncation markers, exit mapping)
- `crates/application/src/quran_graph_export.rs` (Graph JSON assembly, `render_dot`, `render_svg`, truncation banners)
- `crates/quran-graph/src/error.rs` (`GraphError` codes, `render_human`, `next_command`)
- `crates/cli/src/quran.rs` (command tree, `confirm`/APPROVE gate strings)
- `crates/cli/src/lib.rs` (command tree root — confirms `qai quran …` path)
- `.planning/phases/04-quran-graph/04-UI-SPEC.md` (audit baseline)
- `.planning/phases/04-quran-graph/04-{01,02,03,04,05}-SUMMARY.md` (what was built)
- `.planning/phases/04-quran-graph/04-CONTEXT.md` (locked decisions D-11/D-13)

Registry audit: skipped — `components.json` absent, zero third-party registries per UI-SPEC.
