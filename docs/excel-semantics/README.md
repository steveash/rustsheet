# Excel semantics knowledge base

Curated, evidence-backed rules for how Microsoft 365 Excel behaves, especially where
documentation is silent or wrong. **Every rule cites evidence**: an eval case id with oracle
values (`probes/...`, an Excel-authored workbook cell), never memory. Rules marked
**UNVERIFIED** are hypotheses with a pending probe case and must not be implemented until
confirmed.

Files by area: `numbers.md`, `dates.md`, `coercion.md`, `errors.md`, `criteria.md`,
`arrays.md`, `lookup.md`, … (create as needed). Format per rule:

```
## <short rule name>
Rule: one sentence.
Evidence: <suite/case> <Sheet!A1> (oracle: excel-graph 16.0.xxxx) …
Status: VERIFIED | UNVERIFIED (probe pending)
Implemented in: path::fn (once implemented)
```
