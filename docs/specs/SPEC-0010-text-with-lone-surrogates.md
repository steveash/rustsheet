# SPEC-0010 — Engine text that can hold lone surrogates

- **Status:** accepted (on merge of T-1483)
- **Milestone / lane:** M2 / engine (then functions-text, api, research)
- **Tasks:** T-1483 (this spec); implementation tasks in "Rollout" below

## Problem
Excel text is a sequence of UTF-16 code units, and Excel keeps an unpaired ("lone") surrogate
when a cut splits a surrogate pair. Engine text is a Rust `String` (UTF-8), which cannot hold one,
so the engine drops the straddling pair whole (`utf16_slice` in functions/text/util.rs,
`truncate_text` in cast.rs). Decision 0033 accepted that as a deviation; decision 0052 reopened
it: a Rust type limit is not a directive-5 reason. Ten M2 exit cells (decision 0051) fail today:

| Cell | Formula | Excel | Ours |
|---|---|---|---|
| text-surrogate-positions A6 | `LEFT("a😀b",2)&MID("a😀b",3,2)` | "a😀", lone U+DE00, "b" | "a😀b" |
| A11 | `LEN(MID("a😀b",3,2))` | 2 | 1 |
| A12 | `LEN(MID("a😀b",2,1))` | 1 | 0 |
| A29 | `LEN(REPLACE("a😀b",3,1,"x"))` | 4 | 3 |
| A38 | `UNICODE(MID("a😀b",3,1))` | 56832 | `#VALUE!` |
| B10 | `LEN(MID("😀",2,5))` | 1 | 0 |
| concatenate-limit-spill E8 | `LEN(CONCATENATE(REPT("😀",16383),"😀"))` | 32767 | 32766 |
| E9 | `LEN(CONCATENATE(REPT("a",32766),"😀"))` | 32767 | 32766 |
| E10 | `UNICODE(RIGHT(CONCATENATE(REPT("a",32766),"😀"),1))` | `#VALUE!` | 97 |
| E11 | `LEN(REPT("😀",16383)&"😀")` | 32767 | 32766 |

## Goals / non-goals
Goals:
- Engine text values can hold any UTF-16 code-unit sequence, so MID/MIDB/REPLACE/REPLACEB and the
  32,767-unit truncation keep the lone half exactly as Excel does, and the 10 cells pass.
- No cost for well-formed text: no extra allocation, no size growth of `CalcResult` or
  `ArrayNode`, no extra pass on the text hot paths (`&`, CONCATENATE, the text functions).
- The boundaries that can carry a lone surrogate out (xlsx cached values, the public API, the
  Python bindings, the eval harness) carry it exactly.

Non-goals (follow-ups once the probe task below has evidence):
- Lone surrogates *entering* from outside: xlsx constants (`_xD83D_` in sharedStrings or an inline
  string), Python `str` inputs with a lone surrogate, cell inputs through the Rust API. Today's
  behavior stays (the xlsx reader keeps an undecodable `_xD83D_` literally; Rust inputs are `str`
  and cannot hold one; PyO3 rejects such a Python `str`).
- Formula text: the parser reads formulas from `str`, so a string literal cannot hold a lone
  surrogate.
- Functions other than the ones below seeing a lone surrogate exactly (UPPER, TRIM, SUBSTITUTE,
  FIND, comparisons, …): they see the lossy view (below) until probed.

## Excel behavior to match
- MID/REPLACE cut positions count UTF-16 units and keep the lone half (text.md "MID and REPLACE
  positions inside a surrogate pair keep the lone half"; probes/text-surrogate-positions A6, A11,
  A12, A29, A38, B10; excel-graph 2026-10-09, T-0249).
- LEFT/RIGHT count a lone surrogate as one character and do not pair a lone high surrogate with
  the next character (same entry; B6, B7, which pass today because the pair is dropped, and after
  T-1579 keep passing through the lossy view until T-1581 makes LEFT/RIGHT exact).
- Truncation at 32,767 units inside a pair keeps the high surrogate (coercion.md "Truncation inside
  a surrogate pair keeps a lone high surrogate"; probes/concatenate-limit-spill E8-E11, T-0233).
- `UNICODE` of a lone high surrogate is `#VALUE!`, of a lone low one its code (E10; A38).
- Excel writes a lone unit of a cached text value as the `_xHHHH_` escape: the oracle file of
  text-surrogate-positions A6 holds `a😀_xDE00_b`, and our snapshot reader keeps an escape it
  cannot decode (`decode_xlsx_escapes` in rustsheet-xlsx import/shared_strings.rs) literally, so
  the oracle value is the 13-character text `a😀_xDE00_b`.

Unprobed (probe task below): CODE of a lone surrogate (CODE of other non-Windows-1252 characters
is 63, text.md); whether `&` of a lone high and a lone low surrogate forms the pair (UTF-16 says
yes: `EXACT(MID("😀",1,1)&MID("😀",2,1),"😀")`); `=`/`<`/EXACT/MATCH/sort order with lone
surrogates; UPPER/LOWER/PROPER/TRIM/CLEAN; SUBSTITUTE, FIND and SEARCH with a lone half as
find_text (they can split a pair); VALUE; TEXTJOIN and CONCAT truncation; xlsx constants holding
`_xD83D_`.

## Design

### Representation: `Text`, UTF-8 with a boxed exact form for ill-formed text
New module `crates/rustsheet-engine/src/text.rs` (public, re-exported from the crate root):

```rust
/// Excel text: a sequence of UTF-16 code units. Well-formed text (every surrogate paired) is
/// stored as UTF-8; text holding a lone surrogate also keeps its exact units.
#[derive(Clone, Debug, PartialEq, Eq)] // Encode/Decode hand-written, below
pub enum Text {
    Utf8(String),
    Lone(Box<LoneText>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoneText {
    units: Box<[u16]>,  // the exact text; holds at least one lone surrogate
    lossy: String,      // String::from_utf16_lossy(&units): each lone unit as U+FFFD
}
```

Invariant (kept by the constructors, the only way to build a `Text`): `Lone` if and only if the
units are ill-formed UTF-16. So `Text` has one form per value and derived `PartialEq` is value
equality (a lone surrogate never equals a real U+FFFD). `Model::from_bytes` decodes untrusted
bytes, so `Encode`/`Decode` are hand-written, not derived: `Utf8` is encoded as its string,
`Lone` as its units only (`lossy` is not serialized), and decoding a `Lone` rebuilds it through
`from_utf16`, so no byte string can produce a non-normalized `Text` or a `lossy` that disagrees
with `units`.

Why this layout: `Text` is 24 bytes, the size of `String`, and `ArrayNode` stays 24 bytes and
`CalcResult` 40 (measured with `size_of` on stand-ins; the first task asserts both in a test).
Every existing producer moves its `String` in (`Text::from(String)` is a move, no scan: valid UTF-8
is always well-formed UTF-16), and consumers read `&str` through `Deref` at the cost of one match.

API (engine-internal callers, plus the boundaries):
- `From<String>`, `From<&str>`; `Text::from_utf16(Vec<u16>) -> Text` (normalizes to `Utf8` when
  well-formed).
- `Deref<Target = str>`: the **lossy view**, each lone unit as U+FFFD. Documented as such. Code that
  has not been made lone-aware keeps working on it unchanged, and lengths stay right (U+FFFD and a
  lone surrogate are both one UTF-16 unit), so LEN and FIND/SEARCH positions are already exact.
- `len_utf16()`, `has_lone_surrogate()`, `units()` (iterator of `u16`).
- `slice_utf16(start, end) -> Text`: units `[start, end)`, keeping the lone half of a cut pair
  (MID, REPLACE). Replaces `utf16_slice`.
- `chars_lossless()`: iterator of `Char::Scalar(char) | Char::Lone(u16)` (LEFT, RIGHT, UNICODE,
  CODE count and read characters with it).
- `concat(&Text, &Text) -> Text`, `push(&mut self, &Text)` (appends in place, amortized
  linear: stays `Utf8` until a lone unit arrives and re-pairs only at the join, so CONCATENATE's
  argument loop stays O(n)), and `truncate_utf16(&mut self, max)` (keeps a cut pair's high
  half). Replace `concat_text` and `truncate_text` in cast.rs. Concatenation is of unit
  sequences, so a lone high followed by a lone low surrogate becomes the pair (UTF-16's own rule;
  a hand expectation, `EXACT(MID("😀",1,1)&MID("😀",2,1),"😀")`, provisional until T-1582 probes
  it; T-1580 revisits it if T-1582 disagrees).
- `segments()`: iterator of `Segment::Str(&str) | Segment::Lone(u16)` for the boundary writers.
- `into_lossy_string(self) -> String` for consumers that need an owned `String`.

The `Utf8` paths of `concat`, `truncate_utf16` and `slice_utf16` are today's code; the `Lone` paths
work on the unit slice and are rare. Allocation bounds are unchanged: every result is at most
32,767 units (decision 0014 limits still apply before any `Text` is built).

### Where `Text` replaces `String`
- `CalcResult::String(Text)` and `ArrayNode::String(Text)` (the evaluator's values).
- `FormulaValue::Text(Text)` and `SpillValue::Text(Text)` (cached results; bitcode-encoded, so the
  stored model format changes: `Model::to_bytes` is documented as an internal representation,
  and bytes saved by an earlier build fail to load with `from_bytes`'s error, never a panic; the
  task adds a test that old-format bytes are an `Err`).
- Not changed: `shared_strings: Vec<String>` (constants, non-goal), the parser's literals, error
  messages, sheet names, `CellValue` (the engine's legacy getter; lossy).

### Boundaries
- **xlsx writer** (rustsheet-xlsx export): a cached text value is written segment by segment:
  `escape_xml` for each `Str` segment, `_xHHHH_` (upper-case hex, as Excel writes and as
  `escape_xml` already writes control characters) for each `Lone` unit. A literal `_xHHHH_` look-alike in a `Str` segment is
  still written `_x005F_xHHHH_` (xlsx-storage.md). One public, documented helper in
  export/escape.rs formats the escape; the harness reuses it. Only a lone low unit has been
  observed in an Excel file (A6); a lone high one is assumed to be written the same way
  (UNVERIFIED, T-1582).
- **Public API** (crates/rustsheet): `Value::Text(Arc<str>)` stays and holds the lossy view, so no
  consumer silently drops a text value. New accessor
  `Workbook::text_utf16(r) -> Result<Option<Arc<[u16]>>, RefError>`: `Some(units)` exactly when the
  cell's value is text holding a lone surrogate. (A new `Value` variant was rejected: `Value` is
  `#[non_exhaustive]` and existing `_ =>` arms, the harness's among them, would score such text as
  empty.)
- **Python bindings**: when a text value contains U+FFFD, the facade asks `text_utf16` and, on
  `Some`, builds the `str` from the units with `surrogatepass` (Python `str` holds lone
  surrogates), so `wb["A6"]` is `'a😀\ude00b'`. The U+FFFD scan is deliberate: it is a constant
  factor on the O(len) conversion to `str` already paid, cheaper than a second cell lookup on
  every read. The binding's Python surface (method names, types) is unchanged.
- **Eval harness** (rustsheet-eval engine.rs): renders our text with a lone unit as the oracle
  sees Excel's: `Str` segments as-is, `Lone` units as `_xHHHH_` via the xlsx helper. Known
  ambiguity, harness only: Excel text that literally reads `_xDE00_` snapshots the same way.

### Performance
Well-formed text pays one enum match per `Deref` and nothing else; `Text` and `ArrayNode` keep
their sizes. The existing benches (`crates/rustsheet/benches` aggregate, edit) never touch text,
so the first task also adds `crates/rustsheet/benches/text.rs` and records its numbers on main
before any value type changes: recalc of a large column of `=A1&"x"`, of `=MID(A1,2,3)` and
`=LEFT(A1,2)` over a text column, and of a many-argument `CONCATENATE` near the 32,767-unit limit.
T-1576 (the `CalcResult`/`ArrayNode` migration) and T-1580 (`&`, CONCATENATE, truncation) report
that bench before/after in their Outcomes, budget ±3%; aggregate stays a sanity check.

## Alternatives considered
- *WTF-8 bytes (`Vec<u8>`) everywhere.* Exact and 24 bytes, but it has no `&str` view: every one of
  the ~320 `CalcResult::String`/`ArrayNode::String` sites and every `str` algorithm (regex SEARCH,
  case mapping, number parsing) would need an explicit lossy conversion, and the "is it
  well-formed" check is a scan unless a flag grows the type. Rejected for churn and hot-path cost.
- *UTF-16 (`Vec<u16>`) everywhere.* O(1) unit indexing, but doubles ASCII memory and needs a
  conversion for every `str` algorithm. Rejected.
- *`String` plus an `Option<Box<[u16]>>` side field.* Same semantics, but `ArrayNode` grows from 24
  to 32 bytes (every array element). Rejected for the enum above.
- *Encode lone surrogates as private-use characters inside the `String`* (as
  `one_char_per_utf16_unit` does locally for FIND/SEARCH). Collides with real text holding
  U+F0000-U+F07FF, which untrusted workbooks can contain. Rejected.
- *A `CalcResult::LoneText` variant next to `String`.* Every `_ =>` arm would treat such text as
  non-text silently. Rejected.
- *Keep decision 0033 (drop the pair) or substitute U+FFFD.* Not allowed by decision 0052.

## Test plan
- `text.rs` unit and property tests against a `Vec<u16>` model: round trip, `slice_utf16`,
  `concat` (including lone high + lone low), `truncate_utf16`, `chars_lossless`, normalization
  invariant, `size_of` asserts.
- The 10 cells above leave eval/baseline (probes); B6, B7 keep passing; no other eval change.
- xlsx: a writer round-trip test that a lone unit is written as `_xDE00_`.
- API/bindings: `text_utf16` unit test; a Python test that the value is `'a😀\ude00b'`.

## Rollout
Ordered; each S/M; the old `String` paths stay until the task that replaces them.
1. **T-1575** (engine, M): `Text` type and tests, not yet wired; the text bench and its baseline.
2. **T-1576** (engine, M): `ArrayNode::String` and `CalcResult::String` carry `Text` (mechanical,
   lossy view everywhere; zero eval delta; benchmark in the Outcome).
3. **T-1577** (engine, S): `FormulaValue::Text`/`SpillValue::Text` carry `Text`; xlsx writer escape
   of lone units.
4. **T-1578** (api, S): `Workbook::text_utf16`, the harness rendering and the Python bindings.
5. **T-1579** (functions-text, S): MID/MIDB/REPLACE/REPLACEB on `slice_utf16` (A11, A12, A29, B10).
6. **T-1580** (engine, S): `&`, CONCATENATE and truncation on `Text` (E8, E9, E11, A6).
7. **T-1581** (functions-text, S): UNICODE, LEFT/RIGHT on `chars_lossless` (A38, E10; B6, B7 stay).
8. **T-1582** (research, S): probe the unprobed list above; file follow-ups from its findings.

Decision 0033 stays superseded; the excel-semantics entries move from "known gap" to implemented
as tasks 5-7 land.
