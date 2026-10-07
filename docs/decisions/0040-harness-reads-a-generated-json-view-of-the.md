# 0040 — Harness reads a generated JSON view of the function catalog

- **Status:** accepted (2026-10-07, T-0051)
- **Context:** The gap report must count only in-scope catalog functions (decision 0027), but the
  eval crate has no YAML reader and `eval/catalog/functions.yaml` is YAML. `serde_yaml` is archived
  and flagged unmaintained by RustSec (RUSTSEC-2024-0320), and its fork `serde_yml` has its own
  advisory, so a YAML crate would add supply-chain risk to `cargo deny` for one file. tools/ already
  parses the catalog with PyYAML.
- **Decision:** tools/ writes `eval/catalog/scope.json` (`cd tools && uv run rustsheet-catalog-view`):
  each function's name, milestone, scope, reason and oracle, one per line. The harness reads only
  that file. `tools/tests/test_catalog_view.py` fails when the committed view differs from what the
  YAML generates, so any catalog edit must regenerate the view in the same PR.
- **Consequences:** No new Rust dependency. A catalog edit is a two-file change (YAML + view); the
  python test in `make check` catches a forgotten regeneration. If the harness ever needs more of the
  catalog (signatures), extend the view's fields rather than adding a second reader.
