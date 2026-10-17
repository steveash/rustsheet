# 0071 — REGEX functions search on a counted regex-automata engine

- **Status:** accepted (2026-10-17, T-1272)
- **Context:** CLAUDE.md directive 5 needs every REGEX* call (REGEXTEST, REGEXEXTRACT, REGEXREPLACE) bounded,
  and a lifted call searches one pattern over a whole column. The `regex` crate hides which engine runs a
  search. Its lazy DFA quietly falls back to a PikeVM when it thrashes. So a search's cost cannot be predicted
  from the pattern's compiled size: the T-1272 performance review found `a[ab]{100}c|b[ab]{100}c`, a
  64 KiB-tier pattern, running 12.9 s before the size-based charge stopped it. The same charge also stopped
  simple patterns over long columns early.
- **Decision:** the REGEX* functions search with `regex_automata` directly, the crate `regex` is built on
  (functions/text/regex_engine.rs). The search uses a lazy DFA with the `regex` crate's own give-up settings
  (3 cache clears, 10 bytes a state). When it gives up or quits, a PikeVM takes over. Both are compiled from
  one Thompson NFA with the default syntax, leftmost-first semantics and UTF-8 empty-match rules, so matches,
  groups and `$name` interpolation are the `regex` crate's. Each search is charged against a per-call budget
  of 2^30 units (about 1 ns each, release build):
  - the DFA, for what its cache reports it scanned (4 units a byte) and built (8 units a byte of state);
  - the PikeVM, before it runs, for its span times the NFA's state count (5 units a state-byte);
  - 64 units for each search.
  The constants come from measurement: PikeVM about 4.2 ns a state-byte, DFA state building about 6 ns a
  byte, DFA scanning 1-3 ns a byte. Held memory is measured, not estimated from the size tier. A call past a
  budget is `#ERROR!` (DEVIATION, decision 0014; text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX*
  calls are capped").
- **Consequences:** REGEX* no longer uses `regex` or `regex_lite`. XMATCH and static analysis still do.
  wasm32 builds compile `regex-automata` with the `syntax`, `hybrid` and `nfa-pikevm` features. A future
  REGEX* option, such as case sensitivity (T-1360), is set on the `regex_automata` syntax config in
  `Engine::new`. Re-measure the cost constants if `regex-automata` changes major version.
