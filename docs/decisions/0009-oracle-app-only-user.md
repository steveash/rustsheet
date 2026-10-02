# 0009 — Graph oracle: least-privilege app-only access via a dedicated SharePoint site

- **Status:** accepted (2026-10-02)
- **Context:**
  - The Graph client secret lives in the cloud environment, where autonomous agent sessions (which
    process untrusted formulas and LLM output) can read it.
  - `Files.ReadWrite.All` (app-only) would expose every OneDrive and SharePoint site in the
    owner's company.
  - App-only auth has no `/me`, so it needs an explicit drive.
- **Decision:**
  - The default owner setup grants `Sites.Selected` and gives the app write access to one dedicated
    "RustSheet Oracle" SharePoint site.
  - The tool targets that site's document library via `RUSTSHEET_ORACLE_SITE`.
  - Fallbacks, documented with their risk: delegated sign-in, or tenant-wide `Files.ReadWrite.All`
    with `RUSTSHEET_ORACLE_USER`.
  - URLs in errors redact the owner's email.
  - The manual oracle workflow (`.github/workflows/oracle.yml`, protected path) passes the new
    variables through.
- **Consequences:**
  - A leaked secret reaches only the scratch site.
  - Whether Excel workbook endpoints accept `Sites.Selected` app-only tokens is unverified; T-0004
    measures it, and falls back to delegated mode if they don't.
