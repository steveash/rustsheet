# 0001 — Start from a fork of IronCalc

- **Status:** accepted (2026-10-01)
- **Context:** We need an Excel-compatible Rust engine. IronCalc (MIT OR Apache-2.0) already has
  496 functions, a parser with dynamic arrays and LAMBDA, xlsx IO, Python bindings, and 207
  Excel-authored test workbooks. On those workbooks it scores 97.33% bit-exact under our strict
  harness. It lacks a dependency graph, lazy evaluation, compact storage and a transactional API.
- **Decision:** Snapshot-import IronCalc at `e29e7b5` into `crates/rustsheet-engine`,
  `crates/rustsheet-xlsx` and `bindings/python`, renamed and used under Apache-2.0. Drop the
  webapp, wasm and nodejs parts. Do not merge upstream; port fixes deliberately (`UPSTREAM.md`).
- **Consequences:**
  - We inherit lint debt (99 allow entries) and duplication (9,153 duplicated lines), both
    ratcheted down.
  - We must keep the NOTICE and attribution, the Sun fdlibm notices in the Bessel files, and the
    Unicode CLDR notice for locale data.
  - The architecture is replaced incrementally behind the eval gate.
