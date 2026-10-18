# Excel semantics knowledge base

The **canonical list of Excel behaviors we know about and accept**, especially the surprising,
arbitrary, inconsistent or undocumented ones. RustSheet matches every entry exactly (decision 0014).

## Rules
- **Excel is always right.** An entry records what Excel does, not what it should do. Never
  soften, "correct" or delete an entry because the behavior looks like a bug, and never
  implement a nicer variant. If Excel is inconsistent between two cases, record both.
- **Record every quirk you find, when you find it.** Whatever surfaced it (an oracle probe, an
  Excel-authored workbook's cached values, triaging an eval failure, reading a reviewer's
  finding), add or update the entry in the same PR, even if no task implements it yet.
- **Every rule cites evidence**: an eval case id with oracle values (`probes/...`, an
  Excel-authored workbook cell), never memory. Rules marked **UNVERIFIED** are hypotheses with
  a pending probe case and must not be implemented until confirmed. When the oracle disproves a
  hypothesis, rewrite the rule from the evidence and say what was wrong.
- **Excel for the web vs desktop.** When the excel-graph oracle contradicts values desktop Excel
  cached in the same workbook, the entry is `Status: CONFLICTING` with both sets of values, the
  cells stay pending (decision 0019). In Phases 1–3 excel-graph is the oracle of record: a task
  that must choose a value matches the web, and the desktop probe is Phase 4 work (M14, T-1075).
  A desktop-only question never blocks a task or a milestone (decision 0039). Desktop evidence
  already in the repo that cells pass against stays the golden, and non-en-US desktop workbooks
  are no evidence for locale-dependent behavior (decision 0030).
- **Deliberate deviations** are allowed only where CLAUDE.md directive 5 (untrusted input)
  requires one: resource, recursion or allocation limits Excel does not have, no I/O from
  formulas (e.g. WEBSERVICE, external links), no panics. Directive 5 wins over fidelity.
  Decision 0014 is the standing record for these; each one needs an entry here with
  `Status: DEVIATION (decision 0014)` stating Excel's behavior and ours. Example:
  BESSELJ/BESSELY orders above 100,000 return `#NUM!` (`engineering.md`). There is no other
  kind of deviation, and no other decision record can create one (decision 0052). Excel
  behavior RustSheet does not match for any other reason, however costly to match, is a known
  gap: its entry stays `VERIFIED`, says what RustSheet gives under "Ours", and names the task
  that will close it.

Numeric kernels (ERF, GAMMALN, distribution functions, EXP): before trying to match one to the bit, read
`docs/research/excel-kernel-playbook.md`. It lists the patterns Excel's numeric code follows and the methods that
cracked ERF/ERFC.

Files by area: `numbers.md`, `dates.md`, `financial.md`, `coercion.md`, `errors.md`, `criteria.md`,
`arrays.md`, `lookup.md`, `web.md`, `number-formats.md`, `references.md`, `syntax.md`, `logic.md`, `recalc.md`, … (create as needed). Format per rule:

```
## <short rule name>
Rule: one sentence.
Evidence: <suite/case> <Sheet!A1> (oracle: excel-graph 16.0.xxxx) …
Status: VERIFIED | UNVERIFIED (probe pending) | CONFLICTING (web vs desktop) | DEVIATION (decision 0014)
Implemented in: path::fn (once implemented)
```
