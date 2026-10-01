# 0004 — Python bindings first; Java later through a flat FFI facade

- **Status:** accepted (2026-10-01)
- **Decision:**
  - Keep and evolve the PyO3 bindings (`import rustsheet`), upgraded to pyo3 0.29 for
    RUSTSEC-2026-0176/0177.
  - Public API types have no lifetimes or generics, so a future `rustsheet-ffi` crate can expose
    them through UniFFI or Panama FFM to Java.
- **Consequences:** API design reviews check FFI-friendliness. Java work is scheduled for M12.
