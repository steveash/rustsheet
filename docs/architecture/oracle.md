# Excel oracle

**In use:** `excel-graph` is the oracle for now (real Excel, verified live 2026-10-04). The other
backends below are not set up.

The oracle answers "what does Microsoft 365 Excel compute for this workbook?" Every backend
returns a **recalculated xlsx**, and every workbook carries a canary that proves it was
recalculated (see "Is Excel really calculating?" below). Values are extracted with
`rustsheet-eval snapshot`, the single parser, with full 17-digit precision, and stored as
`*.oracle.json` sidecars next to the case (or under the suite's `oracle_dir` for fetched suites).
`fill` lists cases with `rustsheet-eval cases`, skips those with nothing to score or whose cached
values the suite trusts, and leaves pending every cell where the web result contradicts a value
Microsoft Excel cached in the workbook (printed as `CONFLICT`; decision 0019). Results are cached by input sha256 in
`eval/.cache/oracle/<backend>/`, so only new or changed workbooks hit Excel. Agents work
offline against committed sidecars.

| Backend | Where | Fidelity | Notes |
|---|---|---|---|
| `excel-graph` | anywhere (Linux CI, cloud sessions) | Excel for the web (server calc engine) | needs a **work/school** Microsoft 365 tenant; least-privilege setup uses a dedicated SharePoint site (`Sites.Selected`) |
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

## Is Excel really calculating? (read before trusting or changing the oracle)

**Every oracle result is canary-verified.** Before upload, `oracle/canary.py` appends a sheet
`RustSheetCanary` to *each* workbook. Its formulas (`=1+1`, `=RAND()`, `=ROUND(2.675,2)`,
`=SQRT(2)`, plus the en-US locale checks `=DATEVALUE("1/2/2001")` and `=VALUE("1.5")`) carry
deliberately wrong cached values. After the round trip the canary must hold Excel's answers (2,
a value in [0, 1), 2.68, 1.4142135623730951 bit-exact, 1, 1.5); if any planted value
survives, *that workbook* was not recalculated and its results are refused (`fill` writes no
sidecar, `recalc` errors). The canary sheet is stripped before values reach a
sidecar, and cached results in `eval/.cache/oracle/` are re-verified on every read. It rides in
the same workbook on purpose: it costs no extra round trip, and it proves recalculation per
workbook rather than "the service worked once today".

Limits of the canary:
- A trailing sheet is invisible to formulas except via the sheet count, so workbooks using
  `SHEETS()` or `INFO()` are **refused**, never answered (2 of 591 suite workbooks today).
- Excel for the web does not implement `CELL("width")` (it returns `#VALUE!`; desktop returns the
  column width, poi/47804.xlsx), so workbooks using it are **refused** too.
- Malformed packages that cannot take the extra sheet (21 of 591: fuzzer files, backslash part
  names) are refused too; Excel could not open most of them either.
- Workbooks with workbook-structure protection reject the scratch-sheet edit (403) and are
  refused. Some others Excel for the web cannot open at all (`WorkbookNotSupported`, e.g. POI's
  revision-protected workbooks); those need the desktop backend.

**Why this matters: three ways to get cached values that look like Excel's answers.**
1. *Uploading does not recalculate.* SharePoint stores the file as-is (it only adds
   `customXml/` metadata). Values come from Excel only via a workbook session.
2. *Excel saves a session only if it edited the workbook.* After `calculate(FullRebuild)` on a
   file last saved by current Excel (`AppVersion` 16.x, `calcId` 191028/191029), `closeSession`
   persists nothing: Excel computed the right values in-session (the Graph JSON shows them), but
   the file never changes (eTag stays at version 1). Files from other writers (xlsxwriter, older
   Excel) are saved only because Excel upgrades them on open. Observed 2026-10-04 with the
   canary, without the fix:

   | workbook | last saved by | `calcId` | saved? |
   |---|---|---|---|
   | ironcalc `arithmetic.xlsx`, `DAVERAGE.xlsx`, `CELL.xlsx` | Excel 16 / Excel Online | 191028–191029 | **no, stale** |
   | POI `testSharedFormulasRangeSetBlankBug.xlsx` | Excel 16 | 191029 | **no, stale** |
   | POI `workbookProtection-sheet_password-2013.xlsx` | Excel 15 | 152511 | yes |
   | our spec cases (xlsxwriter) | "Excel 12" | 124519 | yes |

   Resetting or removing `calcId` in the upload does *not* make Excel save. The backend therefore
   adds and deletes a scratch sheet in the session **after** calculating; that edit makes
   `closeSession` persist the recalculated workbook, and leaves no trace in it. (Editing before
   calculating also persisted in 3/3 live runs, but an autosave between the edit and the rebuild
   would capture cached values, so the edit goes last; appending and deleting a trailing sheet
   can only re-roll volatiles, which eval never scores.)
3. *A wait that times out silently.* The backend polls the eTag for up to 60 s after
   `closeSession`. Before 2026-10-04 it then downloaded the file anyway, which handed back the
   upload. It now raises `OracleError` instead.

**Typed input** (T-0280). A spec cell `{"entered":"text"}` is written into the open session by a
range `values` PATCH before `calculate`; Excel parses it exactly as typed text (value and number
format) and the PATCH response's `numberFormat` is logged as `ENTERED <sheet>!<a1> ...` on stderr.
Only the value reaches the sidecar: the cached result stores no formats the snapshot reads, and
`CELL("format")` is `#VALUE!` on Excel for the web. Record the logged formats in
`docs/excel-semantics/` with the probe case id.

**Unobservable features** (T-1171). The sidecar holds cell values only, so anything whose effect
is visual is out of reach of excel-graph: conditional-formatting rule results (fills, fonts, and
whether a rule formula such as `=(0.1+0.2-0.3)=0` snaps near zero) cannot be probed. Such questions
go to desktop Excel (M14, Phase 4, decision 0039).

**Unsupported territory.** Microsoft documents the workbook endpoints (sessions, `calculate`) as
**delegated-only**; app-only (client-credentials) use works in this tenant but is undocumented
and may change without notice. Nothing documents when Excel for the web persists a session
either (point 2 was found empirically). This is why the canary exists: if Microsoft changes
any of this, fills fail loudly instead of committing wrong goldens. If app-only stops working,
switch to Fallback A (delegated).

Proof that it is Excel calculating (2026-10-04): with the fixed flow, the downloaded files say
`Application = Microsoft Excel Online, AppVersion 16.0300`, Excel rewrites `calcId` to 191028,
`RAND()` differs on every run, `NOW()` is the run's UTC time, and `ROUND(2.675,2)` is 2.68
(binary-float rounding gives 2.67). `INFO("release")` returns `#VALUE!` (Excel for the web).

## Known divergences: where the oracle is not desktop Excel

Calibration against 206 desktop-Excel-saved ironcalc workbooks (71,765 cells, 99.96% bit-exact;
docs/research/oracle-calibration.md) found where Excel for the web, our oracle, differs (which one
decides goldens when they disagree: decision 0030, desktop Excel wins; in Phases 1–3 decision 0039
matches the web unless desktop evidence already in the repo decides, and desktop probes are Phase 4):

- **References to missing sheets.** `INDIRECT("NoSuchSheet!A1")` is a reference to an empty cell
  in Excel for the web (value 0, `ISREF` TRUE, `ROW` works); desktop Excel returns #REF!. Never
  use oracle values as evidence here (docs/excel-semantics/lookup.md).
- **Last-digit floating point.** In ATAN2, IMARGUMENT, IMREAL/IMSQRT, IMCOS, IMTAN, ERFC,
  BESSELJ/BESSELY, MIRR, IRR and SIN, 18 cells differ from desktop by up to 4 ULPs (11), within 1e-9
  relative (4) or by near-zero residue (3; 2.2e-16 vs 0). Desktop wins (decision 0030): engine
  fits target desktop evidence (Excel-authored workbooks); a known-failure web golden missed
  within 1e-9 is not a fitting target (rule 4). The fill writes the 16.x desktop value for
  differences within 1e-9 (T-0142), except in suites with `use_cached_values = false`, whose caches
  are not desktop evidence (decision 0073); near-zero residues stay pending. Say which one a golden comes from.
- **TEXT formats and CELL (poi, T-0019).** Lowercase-`e` scientific formats give `#VALUE!` on the
  web where Mac Excel cached strings; an elapsed token after a plain `s` gives different hours;
  `CELL("width")` is unimplemented (`#VALUE!`). The fill leaves such cells pending (`CONFLICT`)
  and refuses `CELL("width")` workbooks; docs/excel-semantics/number-formats.md lists them as
  CONFLICTING (decision 0019).
- **JIS unknown, FILTERXML failing on en-US Excel for the web (T-0040).** `JIS` is `#NAME?`,
  bare and as `_xlfn.JIS` (probes/catalog/availability-22 A8, function-name-forms-2 A1/A2);
  `FILTERXML("<a>1</a>","//a")` is `#VALUE!` (probes/catalog/function-name-forms A7), for valid
  XML and XPath (its page says it relies on Windows). Whether desktop Excel differs is unknown.
  The web values are goldens (decision 0039); desktop confirmation is T-1081 (M14). Every other
  in-scope catalog function is recognised (probes/catalog/availability-*). INFO and SHEETS have no
  web values at all: the canary refuses their workbooks (catalog `oracle: none`).
- **Locale is a site setting.** The oracle calculates in the SharePoint site's regional
  settings. The canary pins it to en-US (`DATEVALUE("1/2/2001")-DATEVALUE("1/1/2001")` = 1, not 31; a date difference so that 1904 workbooks pass too, T-1268; `VALUE("1.5")` = 1.5);
  if the site is changed, every fill fails rather than returning another locale's answers.

## Owner setup — Microsoft Graph (one time, ~20 minutes)

**What this does:** it creates a "robot" identity (an *app registration*) in your Microsoft 365
tenant that can upload a workbook, ask Excel to recalculate it, and download the result. Agents use
it to get Excel's real answers.

**Security model (read this):** the robot's secret is stored where autonomous agents can read it,
so the robot gets access to **one dedicated SharePoint site and nothing else** (`Sites.Selected`).
The broader `Files.ReadWrite.All` permission would let it read and write **every user's OneDrive
and every SharePoint site in the company**; don't grant that unless the tenant holds nothing else
of value (fallback B below).

You need a Microsoft 365 **Business** (work/school) account with admin rights. Personal and Family
plans don't work with the Graph Excel API.

1. **Create a dedicated site.**
   1. Open SharePoint (https://www.office.com → SharePoint, or `https://<yourtenant>.sharepoint.com`)
      and click **+ Create site → Team site**.
   2. Name it `RustSheet Oracle`, keep it **Private**, and add no other members.
   3. Note its address, for example `yourtenant.sharepoint.com/sites/RustSheetOracle`.
2. **Register the app.** Go to https://entra.microsoft.com → **Identity → Applications → App
   registrations → + New registration**.
   - Name: `rustsheet-oracle`.
   - Account types: **Accounts in this organizational directory only**.
   - Redirect URI: leave empty.
   - Click **Register**.
3. **Copy two IDs** from the app's **Overview** page: **Application (client) ID** and
   **Directory (tenant) ID**.
4. **Create a secret.** Go to **Certificates & secrets → Client secrets → + New client secret**
   and set it to expire in 12 months. Click **Add**, then copy the **Value** column immediately.
   It is shown only once; the "Secret ID" column is not it.
5. **Add the narrow permission.** Go to **API permissions → + Add a permission → Microsoft Graph →
   Application permissions**.
   - Tick `Sites.Selected` and click **Add permissions**.
   - Click **Grant admin consent** and confirm.
   - On its own this permission grants access to *no* sites. Step 6 opens exactly one.
6. **Allow the app into that one site** (Graph Explorer, about 3 minutes):
   1. Open https://developer.microsoft.com/graph/graph-explorer and sign in as the admin.
   2. Run `GET https://graph.microsoft.com/v1.0/sites/yourtenant.sharepoint.com:/sites/RustSheetOracle`
      and copy the `"id"` from the response (it looks like `yourtenant.sharepoint.com,1234…,5678…`).
      If you get a permission error, open **Modify permissions** and consent to `Sites.FullControl.All`
      for Graph Explorer **for yourself only**. Do *not* tick "consent on behalf of your organization".
      Then run it again.
   3. Switch the method to **POST**. Set the URL to
      `https://graph.microsoft.com/v1.0/sites/<that id>/permissions` and the request body to:
      ```json
      {"roles": ["write"],
       "grantedToIdentities": [{"application": {"id": "<Application (client) ID>", "displayName": "rustsheet-oracle"}}]}
      ```
      Run it. You should get **201 Created**.
7. **Give the values to the agents.** In Claude Code on the web, click the environment name in the
   session title bar, then **Edit → Environment variables**, and add:
   ```
   RUSTSHEET_ORACLE_TENANT_ID=<Directory (tenant) ID>
   RUSTSHEET_ORACLE_CLIENT_ID=<Application (client) ID>
   RUSTSHEET_ORACLE_CLIENT_SECRET=<secret Value>
   RUSTSHEET_ORACLE_SITE=yourtenant.sharepoint.com/sites/RustSheetOracle
   ```
   Save. New sessions, including the routines, pick these up.
   - Never paste these values into a chat.
   - If the environment uses a restricted network policy, allow `login.microsoftonline.com`,
     `graph.microsoft.com` and `*.sharepoint.com`.
8. **Done.** The next steward run (or any session) checks the setup with
   `uv run rustsheet-oracle status` and runs the first Excel calibration (task T-0004). If the
   calibration shows the app is refused, T-0004 records why and falls back to option A below.

**Hygiene:**
- Rotate the secret before it expires (12 months).
- If it may have leaked, delete it under Certificates & secrets; the robot loses all access
  immediately.
- If you granted `Files.ReadWrite.All` earlier, remove it: API permissions → `…` → Remove
  permission.
- After step 6, remove Graph Explorer's `Sites.FullControl.All` consent: https://myapps.microsoft.com
  → Graph Explorer → Manage your application → revoke, or Entra → Enterprise applications → Graph
  Explorer → Permissions.

**Fallback A — delegated sign-in.** This is officially the supported mode for workbook endpoints.
The robot acts as you and can reach your files only.
- Enable **Authentication → Allow public client flows**.
- Add the **Delegated** permissions `Files.ReadWrite` and `offline_access`.
- Leave `RUSTSHEET_ORACLE_CLIENT_SECRET` unset, and run `uv run rustsheet-oracle login` once to
  sign in with a device code.
- The token cache is written to `~/.config/rustsheet-oracle/msal_cache.json`. For cloud sessions,
  base64 it into `RUSTSHEET_ORACLE_MSAL_CACHE_B64`. Refresh tokens rotate and expire after 90 days
  without use.

**Fallback B — tenant-wide app-only (not recommended).** Grant `Files.ReadWrite.All`
(Application) instead of `Sites.Selected`, and set `RUSTSHEET_ORACLE_USER=<your work email>`
instead of `_SITE`. Scratch files then go to that user's OneDrive. This gives the secret access to
all company files.

**GitHub Actions:** only the manual-dispatch workflow `.github/workflows/oracle.yml` needs
secrets; the routines don't. To use it, create a GitHub environment named `oracle` (repo
Settings → Environments) with the same names as secrets.

Variables read by the tool:
- `RUSTSHEET_ORACLE_TENANT_ID`, `_CLIENT_ID`, `_CLIENT_SECRET`
- one of `_SITE`, `_DRIVE_ID` or `_USER`
- `_MSAL_CACHE_B64` (delegated mode)
- `_FOLDER` (default `/rustsheet-oracle`)

## Owner setup — Windows desktop backend (later, optional)
Not planned soon. Setting it up starts Phase 4 (M14 in docs/milestones.md; decision 0039). Until
then nothing waits for it: excel-graph is the oracle of record for Phases 1–3.
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
   **Yes**, 17 significant digits (`SQRT(2)` = 1.4142135623730951 in the file; the Graph JSON
   rounds to 15, which is why we never use it). The canary checks this on every workbook.
2. How long is the delay before the downloaded file reflects the recalculation (eTag polling)?
   **Usually none**: the eTag has moved by the first poll after `closeSession`; a full round trip
   takes ~10 s. But see "Is Excel really calculating?": without an edit Excel may never save.
3. Does app-only auth work for workbook endpoints in this tenant? **Yes** (2026-10-04, `Sites.Selected`
   on a dedicated site), although Microsoft documents those endpoints as delegated-only.
4. Calibration: on how many ironcalc-calc cells do Graph and cached values disagree? **29 of
   71,765** (2026-10-04): 7 are day-first date text in the workbooks' authoring Excel (the
   oracle is right for en-US), 4 are the missing-sheet `INDIRECT` divergence, 18 are last-digit
   floating point. See "Known divergences" and docs/research/oracle-calibration.md.
