# Excel oracle

The oracle answers "what does Microsoft 365 Excel compute for this workbook?" Every backend
returns a **recalculated xlsx**. Values are extracted with `rustsheet-eval snapshot`, the single
parser, with full 17-digit precision, and stored as `*.oracle.json` sidecars next to the case.
Results are cached by input sha256 in `eval/.cache/oracle/<backend>/`, so only new or changed
workbooks hit Excel. Agents work offline against committed sidecars.

| Backend | Where | Fidelity | Notes |
|---|---|---|---|
| `excel-graph` | anywhere (Linux CI, cloud sessions) | Excel for the web (server calc engine) | needs a **work/school** Microsoft 365 tenant; personal/family plans are not supported by the Graph Excel API |
| `excel-desktop` | Windows machine/VM with Excel, interactive session | highest (desktop-only functions: FILTERXML, WEBSERVICE, CELL/INFO) | COM via pywin32; run as a GitHub self-hosted runner labelled `excel-oracle` |
| `libreoffice` | anywhere | approximate (~97–99% agreement) | triage only; never produces sidecars |
| *cached values* | — | exact for Excel-authored files | used automatically by `xlsx` suites with `use_cached_values` |

CLI (from `tools/`):
```
uv run rustsheet-oracle status                         # which backends are usable
uv run rustsheet-oracle login                          # Graph delegated: one-time device-code login
uv run rustsheet-oracle fill -b excel-graph --suite probes
uv run rustsheet-oracle recalc -b excel-graph some.xlsx
```
CI: `.github/workflows/oracle.yml` (manual dispatch → PR with new sidecars).

## Owner setup — Microsoft Graph (one time, about 20 minutes)

0. **Account check.** The Graph Excel API only works with files in OneDrive for Business or
   SharePoint. If your Microsoft 365 is a Personal or Family plan (an @outlook.com, @hotmail.com
   or @live.com sign-in), get a work/school tenant. *Microsoft 365 Business Basic* is the
   cheapest plan that works; you become its admin.
1. **Register an app:** https://entra.microsoft.com → Identity → Applications → App
   registrations → New registration.
   - Name: `rustsheet-oracle`.
   - Supported account types: "Accounts in this organizational directory only".
   - Note the **Application (client) ID** and **Directory (tenant) ID**.
2. **Choose an auth mode:**
   - **App-only** (best for unattended CI; workbook endpoints are officially "not supported" but
     widely reported to work, so test it first):
     1. Under Certificates & secrets, create a new client secret.
     2. Under API permissions → Microsoft Graph → Application, add `Files.ReadWrite.All`, then
        **Grant admin consent**.
     3. Find your drive id: in Graph Explorer, `GET https://graph.microsoft.com/v1.0/users/<you@tenant>/drive`.
   - **Delegated** (officially supported):
     1. Under Authentication, set "Allow public client flows" to Yes.
     2. Under API permissions → Microsoft Graph → Delegated, add `Files.ReadWrite`,
        `offline_access` and `User.Read`, then grant admin consent.
     3. Run `uv run rustsheet-oracle login` once on a trusted machine and sign in with the device code.
     4. The token cache lands in `~/.config/rustsheet-oracle/msal_cache.json`. For CI, base64 it
        into the secret `RUSTSHEET_ORACLE_MSAL_CACHE_B64`. Refresh tokens rotate and last 90 days
        while used.
3. **Store the configuration.** Never paste it into chat.
   - **Claude Code cloud environment:** session title bar → environment → Edit → environment
     variables.
   - **GitHub:** repo Settings → Environments → create `oracle` → add the same names as secrets.

   | Variable | Needed for |
   |---|---|
   | `RUSTSHEET_ORACLE_TENANT_ID` | both modes |
   | `RUSTSHEET_ORACLE_CLIENT_ID` | both modes |
   | `RUSTSHEET_ORACLE_CLIENT_SECRET` | app-only |
   | `RUSTSHEET_ORACLE_DRIVE_ID` | app-only |
   | `RUSTSHEET_ORACLE_MSAL_CACHE_B64` | delegated |
   | `RUSTSHEET_ORACLE_FOLDER` | optional (default `/rustsheet-oracle`) |
4. **Network:** allow `login.microsoftonline.com`, `graph.microsoft.com` and `*.sharepoint.com`
   in the environment's network policy. The first two were reachable at inception.
5. **Smoke test:** `uv run rustsheet-oracle status`, then
   `uv run rustsheet-oracle fill -b excel-graph --suite probes --limit 1`.

## Owner setup — Windows desktop backend (later, optional)
- A Windows 11 machine or VM with Microsoft 365 Apps, signed in, with auto-logon to a dedicated
  user. Excel must run in an interactive session, not as a service.
- Install Python 3.11+, `uv`, Rust and `pip install pywin32`. Register a GitHub self-hosted runner
  as a console app with label `excel-oracle`.
- Add the working folder as a Trusted Location, and disable Protected View, add-ins and update prompts.
- Licensing: on Windows Server or multi-session hosts, Excel requires shared computer activation
  (M365 Business Premium/E3/E5). A single-user Windows 11 VM avoids this.
- Microsoft does not support unattended Office automation. The backend therefore uses one Excel
  process per job, a watchdog, and restarts on hang.

## Open questions (first oracle session, task T-0004)
1. Does Graph persist full-precision values in the downloaded file after `calculate(FullRebuild)`?
   This needs measuring; we never rely on Graph's JSON values.
2. How long is the delay before the downloaded file reflects the recalculation (eTag polling)?
3. Does app-only auth work for workbook endpoints in this tenant?
4. Calibration: on how many ironcalc-calc cells do Graph and cached values disagree?
