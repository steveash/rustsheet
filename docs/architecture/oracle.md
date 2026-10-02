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

## Owner setup — Microsoft Graph (one time, ~15 minutes)

**What this does:** it creates a "robot" identity (an *app registration*) in your Microsoft 365
tenant that is allowed to put a workbook into your OneDrive, ask Excel to recalculate it, and
download the result. Agents then use it to get Excel's real answers. Scratch files go in a
`rustsheet-oracle` folder of the OneDrive belonging to the work account you name, and are
deleted after each run.

You need: a Microsoft 365 **Business** (work/school) account with admin rights. Personal or
Family plans don't work with the Graph Excel API.

1. **Open the Entra admin center.** Go to https://entra.microsoft.com and sign in with your
   Microsoft 365 business admin account.
2. **Create the app.** In the left menu go to **Identity → Applications → App registrations**,
   then click **+ New registration**.
   - Name: `rustsheet-oracle`.
   - Supported account types: **Accounts in this organizational directory only**.
   - Redirect URI: leave empty.
   - Click **Register**.
3. **Copy two IDs.** The app's **Overview** page shows **Application (client) ID** and
   **Directory (tenant) ID**. Copy both.
4. **Create a secret.** In the left menu go to **Certificates & secrets → Client secrets →
   + New client secret**.
   - Description: `rustsheet`. Expires: 12 months.
   - Click **Add**, then immediately copy the **Value** column. It is shown only once; the
     "Secret ID" column is *not* what you want.
5. **Give it file access.**
   1. In the left menu go to **API permissions → + Add a permission → Microsoft Graph →
      Application permissions**.
   2. Search for `Files.ReadWrite.All`, tick it, and click **Add permissions**.
   3. Click **Grant admin consent for <your org>** and confirm. The status column turns green.
6. **Give the values to the agents.** In Claude Code on the web, click the environment name in
   the session title bar, then **Edit**. Under **Environment variables**, add:
   ```
   RUSTSHEET_ORACLE_TENANT_ID=<Directory (tenant) ID from step 3>
   RUSTSHEET_ORACLE_CLIENT_ID=<Application (client) ID from step 3>
   RUSTSHEET_ORACLE_CLIENT_SECRET=<secret Value from step 4>
   RUSTSHEET_ORACLE_USER=<your work email, e.g. you@yourcompany.com>
   ```
   Save. New sessions, including the scheduled routines, pick these up. Never paste these values
   into a chat.
7. **Done.** The next steward run (or any session) checks the setup with
   `uv run rustsheet-oracle status` and runs the first Excel calibration (task T-0004).

GitHub Actions secrets (`oracle` environment) are only needed if you want to use the
manual-dispatch workflow `.github/workflows/oracle.yml`. The routines don't need them.

**If app-only access is refused** (Microsoft documents workbook endpoints as delegated-only; most
tenants allow app-only in practice), T-0004 switches to delegated mode:
- Enable **Authentication → Allow public client flows**.
- Add the **Delegated** permissions `Files.ReadWrite` and `offline_access`.
- Run `uv run rustsheet-oracle login` once to sign in with a device code.

Variables read by the tool: `RUSTSHEET_ORACLE_TENANT_ID`, `_CLIENT_ID`, `_CLIENT_SECRET`, `_USER`
(or `_DRIVE_ID`), `_MSAL_CACHE_B64` (delegated mode), and `_FOLDER` (default `/rustsheet-oracle`).

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
