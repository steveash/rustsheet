# 0099 — regex-syntax as a direct dependency for the PCRE2 parser

- **Status:** accepted (2026-10-22, T-1973)
- **Context:** SPEC-0013 replaces the `regex` crate's pattern dialect with our own PCRE2 parser
  (`functions/text/pcre/`). The parser builds a `regex_syntax::hir::Hir` for regex-automata's Thompson
  compiler (`build_from_hir`). It also uses regex-syntax's Unicode class tables (`\p{..}`, `\w`, the POSIX
  classes) and `case_fold_simple`, so both engines share one class and one case-folding definition.
  regex-automata does not re-export the `Hir` types. `regex-syntax` 0.8 was already a direct dependency of
  rustsheet-engine for the T-1903 POSIX rewrite (`regex_posix.rs`), which this parser replaces. It was also
  already in Cargo.lock through `regex` and regex-automata's `syntax` feature.
- **Decision:** keep `regex-syntax = "0.8"`, which is the range regex-automata 0.4 uses, as a direct
  dependency of rustsheet-engine for the `Hir`, class and case-folding API. Its own pattern parser is used
  only to build class tables from fixed spellings and `\p{name}`, and in the differential unit test. It is
  not used to parse user patterns.
- **Consequences:** `cargo tree -d` must keep showing one `regex-syntax`. A regex-syntax bump moves with
  regex and regex-automata, never alone. The edge adds no code to the binary, because the crate is already
  linked.
