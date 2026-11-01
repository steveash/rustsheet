# 0221 — The FORECAST.ETS family ships an AAA model adapted from gridcraft before Excel evidence exists (owner ruling)

- **Status:** accepted (2026-11-01, T-3599; owner ruling "Owner ruling: implement now", session
  https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr). Applies to T-3599, T-3600 and T-3601.
- **Context:**
  - The gridcraft head-to-head (docs/research/gridcraft-head-to-head-2026-10-31.md, on branch
    `claude/wonderful-cray-9q8ljx`) found gridcraft ahead on the FORECAST.ETS family (SEASONALITY 81% to 9%) and named
    its implementation (`crates/functions/src/ets.rs`) as worth reading, subject to a licence and provenance review. The
    owner asked for that review and, if the licence is permissive, for RustSheet's own versions of FORECAST.ETS and
    FORECAST.ETS.SEASONALITY.
  - Licence and provenance (checked at gridcraft 67e39fae): the workspace is `MIT OR Apache-2.0`, with both texts in
    the repository and the functions crate inheriting the workspace licence. `ets.rs` cites only a public textbook,
    Hyndman, Koehler, Ord & Snyder, "Forecasting with Exponential Smoothing" (2008). Its history shows it written in
    gridcraft by its contributors (#314, #322, #1042, #2412). gridcraft's AGENTS.md makes the project clean-room: it
    never reads inside Excel and never copies GPL, AGPL or LGPL code.
  - Evidence: Excel for the web never computes the family. Every valid call is `#N/A` (statistics.md "FORECAST.ETS
    family: excel-graph never computes it"). None of the 1,722 Excel-desktop workbooks in calipers, corpus-bench and
    poi uses it, and the `excel-desktop` oracle does not exist yet (T-1553). gridcraft's lead was therefore measured
    against web `#N/A`s, not against Excel's numbers. gridcraft's own documentation says its fitted parameters can
    differ from Excel's. Until now directive 1 kept the family `#N/IMPL!` (T-2425).
  - The owner was told all of this and chose to implement now rather than wait for desktop values.
- **Decision:**
  - The FORECAST.ETS family computes the additive-error, additive-trend, additive-seasonality (AAA) model as gridcraft
    implements it. That covers the series building, timeline step, duplicate aggregation, data completion, season
    detection, fit, forecast, interval and statistics, adapted with attribution. The functions arrive in order:
    SEASONALITY (T-3599), FORECAST.ETS (T-3600), CONFINT and STAT (T-3601).
  - This is an owner-ruled exception to directive 1: what these functions compute is unverified against Excel.
    statistics.md marks every such rule UNVERIFIED, and the code says so where it computes.
  - Excel evidence still wins. The argument step evidenced on excel-graph (T-2424) is unchanged and runs first. Once
    desktop values exist (T-1553), T-2425 replaces whatever they contradict.
  - The adapted code keeps gridcraft's MIT notice in the file that holds it, as the statrs ports do. NOTICE lists it.
- **Consequences:**
  - FORECAST.ETS.SEASONALITY (T-3599): `make eval-baseline` fixes 275 cells (gen-m9 274, probes 1) with no
    regression. Every one is a SEASONALITY call whose `values` and `timeline` differ in size. That is `#N/A` in
    Microsoft's documentation and on excel-graph, so each matches Excel's own `#N/A`. None is evidence for the model.
    Calls that now compute a number stay known failures against the web's `#N/A`, owned by T-2425 as before.
  - FORECAST.ETS (T-3600): `make eval-baseline` fixes 169 more cells (gen-m9 168, probes 1) with no regression. Each
    expects Excel's `#N/A`, and each matches through the same size rule (130 top-level FORECAST.ETS calls, 39 nested
    in another function). None is evidence for the fit.
  - FORECAST.ETS.CONFINT and FORECAST.ETS.STAT (T-3601): `make eval-baseline` fixes 318 more cells (gen-m9 317, probes
    1) with no regression. Every one expects Excel's `#N/A` and matches through the same size rule (STAT 224 cells, 188
    of them top-level; CONFINT 94, 78 top-level). None is evidence for the interval or the statistics. With T-3601
    no member of the family is `#N/IMPL!` any more.
  - Directive 5 deviations, listed in statistics.md: season detection is `#NUM!` past 2^27 multiply-adds (a series of
    16,385 points or more), `values` or `timeline` above 200,000 cells is `#NUM!`, and one call's detections and fits
    share a work budget of 2^30 (T-3600).
  - T-2425 and T-1553 keep their scope: desktop values, then matching them. Their M14 milestone and campaigns are
    unchanged.
  - Reviewers check the attribution and the UNVERIFIED marking. They do not ask for Excel evidence that this ruling
    has waived for the model's values. They do ask for it for anything that contradicts existing web evidence.
