# SPEC-0008 — Per-workbook clock for NOW and TODAY

- **Status:** accepted (on merge of T-1478)
- **Milestone / lane:** M2 / api (engine part: engine)
- **Tasks:** T-1478 (this spec), T-1479 (engine), T-1480 (facade and Python)

## Problem
M2's scope includes "TODAY/NOW (deterministic clock injection)" (docs/milestones.md). Today only the
test harness can make the clock deterministic, and only by a process-wide trick:

- Every reader of the time calls the free function `rustsheet_engine::get_milliseconds_since_epoch`
  (model.rs). Under `cfg(test)` or the `mock_time` cargo feature it reads a thread-local `MOCK_TIME`
  (mock_time.rs, default 1,667,906,008,578 ms = 2022-11-08T11:13:28.578Z). Otherwise it reads
  `SystemTime` and `expect`s that the system time is after 1970, which is a panic path (directive 5).
- The readers are:
  - NOW and TODAY (`Model::now_in_timezone_arg` → `tz::excel_serial_for_now`);
  - year-less date text such as `"7/1"`, read in the current year (`Model::current_year` →
    `tz::current_year`; docs/excel-semantics/dates.md "Year-less text");
  - the creation timestamp of a new workbook (new_empty.rs);
  - `docProps/core.xml` `dcterms:modified` on export (rustsheet-xlsx export/mod.rs);
  - the wasm variants in tz/browser_tz.rs.
- The facade's only control is `Workbook::pin_clock_from_metadata`, behind the `mock_time` feature
  ("harness only", SPEC-0001). It sets the thread-local from the IronCalc `METADATA`/`NOW`
  convention. The Python binding has no control at all.

As a result:
- an embedder cannot pin the clock without rebuilding with a cargo feature;
- two workbooks on one thread cannot have different clocks, and a workbook moved to another thread
  (the facade `Workbook` is `Send`) silently changes clock;
- a saved file's bytes depend on the wall clock.

## Goals / non-goals
Goals:
- Each `Workbook` owns its clock: the system time (the default, today's behavior) or a fixed
  instant. Every in-engine reader of the time uses it.
- The Rust facade and Python can set the clock at construction and change it later. A change is an
  edit, so it is atomic and reported in the `ChangeSet`.
- The thread-local mock and the `mock_time` feature are retired, and the harness results do not
  change.
- No panic on any clock value or on a bad system clock (directive 5).

Non-goals:
- RAND, RANDBETWEEN and RANDARRAY seeding (not M2 scope).
- A "ticking" clock (a fixed offset from the system time) or a user callback. `Fixed` covers
  determinism. A callback would put embedder code inside evaluation, which directive 5 rules out
  (evaluation performs no I/O and runs no foreign code).
- One instant per recalc. Today each NOW call reads the system clock on its own, so two NOW cells
  in one recalc can differ by a millisecond. Whether Excel reads its clock once per recalc is
  unprobed. Under `Fixed` the question does not arise, and `System` keeps today's per-call
  reading.
- The timezone model. `WorkbookOptions.timezone` and the engine's `NOW(tz)`/`TODAY(tz)` extension
  are unchanged.

## Excel behavior to match
- Excel has no clock setting: NOW and TODAY read the machine's local time at each recalc, and
  both are volatile (Microsoft's NOW/TODAY documentation; probes/volatile-invariant shows they
  return numbers and that TODAY is within NOW's day, numbers.md). The engine's "local time" is
  the workbook timezone, `"UTC"` by default, not the host's zone. That difference predates this
  spec, and `System` keeps it. A fixed clock is an embedder feature, not Excel behavior. Under it, NOW and
  TODAY must still give exactly what Excel gives at that instant:
  - NOW keeps sub-second precision (dates.md "NOW() keeps sub-second precision", ironcalc-calc
    DATE_AND_TIME/NOW_TODAY.xlsx);
  - TODAY is NOW's day, a whole number;
  - RAND, NOW and TODAY return numbers (numbers.md, probes/volatile-invariant).
- The harness reproduces Excel's cached NOW/TODAY values by pinning the clock to the
  `METADATA`!B `NOW` serial (an IronCalc convention, read as a UTC serial). That must keep working
  unchanged: 0 eval fixes and 0 regressions, including the volatile-skip and run-invariant scoring
  (T-0005, T-1283; crates/rustsheet-eval/src/volatile.rs `pinned_clock_serial`).

## Design

### Engine (T-1479)
New module `crates/rustsheet-engine/src/clock.rs`:
```rust
/// Where a workbook reads the current time from (NOW, TODAY, year-less date text, timestamps).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Clock(Kind);
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Kind { #[default] System, Fixed(i64) }

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("clock instant {0} ms is outside 1900-01-01..=9999-12-31 UTC")]
pub struct ClockError(pub i64);

impl Clock {
    pub const fn system() -> Self;
    /// A fixed instant, in milliseconds since 1970-01-01T00:00:00Z.
    pub fn fixed_unix_millis(ms: i64) -> Result<Self, ClockError>;
    /// The fixed instant, or `None` for the system clock.
    pub fn fixed(self) -> Option<i64>;
    /// The current instant in ms since the Unix epoch.
    pub fn now_unix_millis(self) -> i64;
}
```
- **Validation.** `fixed_unix_millis` accepts only instants whose UTC date is in Excel's calendar,
  1900-01-01 through 9999-12-31. That is -2,208,988,800,000 ≤ ms ≤ 253,402,300,799,999. Every
  reader can then convert the instant without an error path, `DateTime::from_timestamp` included,
  so export never fails on a fixed clock. A timezone offset can move the local date one day
  outside that range. NOW then gives a number or the existing `#ERROR!` "Invalid date" result,
  and never panics. Excel cannot reach such a date, so `#ERROR!` there is no parity claim. The
  type cannot be built invalid because its field is private. The test plan
  pins both bounds in a non-UTC timezone.
- **Serials before 1900-03-01.** `excel_serial_for_now` must number days as DATE does, through
  the same conversion (`convert_to_serial_number` in formatter/dates.rs, made `pub(crate)` and
  reused; no second copy). Excel's fictitious 1900-02-29 is serial 60, so 1900-01-01 is serial 1
  and 1900-02-28 is 59 (dates.md "Serial 60 is the fictitious 1900-02-29", VERIFIED for DATE).
  - The current `num_days_from_ce() - EXCEL_DATE_BASE` gives one more than that before
    1900-03-01. T-1479 fixes it.
  - NOW in early 1900 cannot be probed, since Excel's clock is the host's, so this is inferred
    from the DATE rule.
- **System time.** `now_unix_millis` on `System` reads `SystemTime` (or `js_sys::Date` on
  wasm32) and clamps the result to the same range as `Fixed`, so the "no error path" guarantee
  holds for both kinds.
  - A system time before 1970 is negated from the `duration_since` error's duration.
  - Every conversion is checked: `Duration::as_millis` (a `u128`) goes through `i64::try_from`,
    and the negation through `checked_neg`. On failure the value saturates to the nearer bound.
  - A non-finite wasm `Date::now()` gives the lower bound.
  - One internal helper, `fn system_unix_millis(t: SystemTime) -> i64`, does the conversion and
    the clamp. It is the test seam.
  - This replaces the `expect`. A host clock outside 1900..=9999 is a misconfiguration, and
    clamping it is a directive-5 choice. Excel itself cannot run with such a clock, so no Excel
    behavior is involved.
- **Model.**
  - `Model` gets a field `clock: Clock`, with `Model::clock(&self) -> Clock` and
    `Model::set_clock(&mut self, Clock)`. `set_clock` does not evaluate, unlike `set_timezone`.
    The facade decides when to recalc.
  - All readers go through `Model::now_unix_millis(&self)`.
  - `tz::excel_serial_for_now`, `tz::current_year` and their browser_tz twins take the
    milliseconds as a parameter instead of reading the clock.
  - The xlsx exporter reads `model.clock().now_unix_millis()`.
  - `Model::new_empty` keeps its signature (system clock). A sibling
    `Model::new_empty_with_clock(.., clock: Clock)` stamps `created` with the given clock.
- **Retired.**
  - The free function `get_milliseconds_since_epoch` and `mock_time.rs` (the thread-local) are
    removed.
  - The engine and xlsx crates' `mock_time` cargo features are removed.
  - The thread is no longer part of a workbook's state.
- **Engine tests.** Under `cfg(test)` the engine read the thread-local, which defaulted to
  1,667,906,008,578 ms. Tests now get that instant from their constructor helpers
  (`test::util::new_empty_model`, `test_locale::new_empty_model`, and the xlsx crate's
  test-workbook loader) as `Clock::fixed_unix_millis(1_667_906_008_578)`. test_now.rs and
  test_today.rs call `model.set_clock(..)` where they call `set_mock_time` today. There is no
  `cfg(test)` default: `Clock::default()` is the system clock everywhere.
- **METADATA pinning.**
  - The xlsx crate's `mock_time` module becomes an ungated `metadata_clock` module.
    `get_metadata_timestamp(&Model) -> Option<i64>` is unchanged, and
    `pin_clock_from_metadata(&mut Model) -> bool` sets `Clock::fixed_unix_millis(ms)`.
  - A timestamp outside the range does not pin and returns false.
  - Without a timestamp it sets nothing. The system-time fallback goes away, along with its
    `unwrap`.
  - compare.rs (`test_file`) calls `pin_clock_from_metadata`.

### Facade (T-1479 for the harness hook, T-1480 for the public surface)
```rust
pub use rustsheet_engine::clock::{Clock, ClockError};   // like DateSystem

pub struct WorkbookOptions { /* existing fields */ pub clock: Clock /* default Clock::system() */ }

impl Workbook {
    /// The workbook's clock.
    pub fn clock(&self) -> Clock;
    /// One-op `edit`: sets the clock, recalculates once, and reports the changed values.
    pub fn set_clock(&mut self, clock: Clock) -> Result<ChangeSet, EditError>;
}
impl Txn<'_> {
    /// Sets the clock used by this transaction's recalc and every later one.
    pub fn set_clock(&mut self, clock: Clock) -> Result<(), EditError>;
}
```
- **Construction.** `Workbook::new`, `open` and `from_xlsx_bytes` apply `opts.clock` before their
  first recalc, so the load-time recalc already uses a fixed clock. `new` builds through
  `new_empty_with_clock`.
- **Edits.**
  - `Txn::set_clock` is an undoable op: the undo log records the previous clock, and a rolled-back
    edit restores it.
  - Like every other edit, the op recalculates once when the transaction ends.
  - The `ChangeSet` reports the NOW/TODAY cells, their dependents, and cells reading year-less
    date text whose value changed. This needs no new diff mechanism, because the existing diff
    covers every formula cell (SPEC-0001).
  - `inputs` and `sheets` stay empty for a clock-only edit.
  - `set_clock` with the current clock still recalculates. It is one edit like any other, and
    RAND cells may change.
- **Volatility.** NOW and TODAY stay volatile: they are recomputed on every recalc. Under a fixed
  clock they give the same value every time, so a `ChangeSet` reports them only when the clock
  changes. SPEC-0001's "Volatile cells … are reported in every `ChangeSet`" bullet gets that
  qualification. Volatility and M3 dirty tracking still treat a clock change as dirtying every
  cell that reads the clock. Today that is every formula cell, since the recalc is full.
  - Clock readers cannot be found from the formula. Year-less date text (`A1+0`,
    `DATEVALUE(A1)`) reads the clock only when its text has no year. So until M3 has a
    conservative mechanism, a clock change dirties every formula cell. Such a mechanism could
    flag a cell at evaluation time when it calls `Model::now_unix_millis`.
  - M3's tests include a year-less text cell that is not NOW/TODAY and changes under
    `set_clock`.
  - Making NOW/TODAY non-volatile while the clock is fixed is a possible M3 optimization, but
    only together with that invalidation on `set_clock`.
- **Saving.** `to_xlsx_bytes` and `save_xlsx` stamp `dcterms:modified` from the workbook's clock.
  With a fixed clock, saving the same workbook twice gives identical bytes.
- **Harness hook.**
  - The facade's cargo feature `mock_time` is renamed `metadata_clock`, because it no longer mocks
    anything. It keeps `Workbook::pin_clock_from_metadata(&mut self) -> bool`, now built on the
    engine clock. It still exists only for the harness, which pins to IronCalc's METADATA
    convention.
  - Changing it from `&self` to `&mut self` follows from the clock being workbook state.
- **Eval adapter** (crates/rustsheet-eval/src/engine.rs).
  - `evaluate_xlsx` loads with `recalc: false` and `clock: Clock::fixed_unix_millis(1_667_906_008_578)`.
    That is the instant every eval thread had through the thread-local default.
  - It then calls `pin_clock_from_metadata()` and recalculates, as today.
  - `evaluate_spec` builds its `WorkbookOptions` with the same fixed instant. Today spec cases get
    it through the thread-local, so their results stay the same.
  - A named constant `HARNESS_CLOCK_MS` in the adapter holds the instant, with a comment citing
    this spec.
  - rustsheet-eval's Cargo.toml enables `metadata_clock` on the facade only. It drops its
    `mock_time` features on the engine and xlsx dependencies.
  - The xlsx crate's self dev-dependency (`features = ["mock_time"]`, Cargo.toml) goes too.

### Python (T-1480)
```python
wb = rs.Workbook(date_system="1900", clock=None)      # clock: int ms since the Unix epoch (UTC) or None = system
wb = rs.Workbook.open(path, recalc=True, clock=None)  # also from_bytes(data, recalc=True, clock=None)
wb.clock                     # int | None
wb.clock = 1_700_000_000_000 # one edit: recalculates, sets wb.changes; None restores the system clock
```
- An int outside the range raises `ValueError` (from `ClockError`). A `bool` (an `int`
  subclass in Python) raises `TypeError` explicitly, and so does any other non-int, non-None value
  raises `TypeError`. The setter releases the GIL during the recalc, as `edit` does.
- `datetime` is not accepted in v0 (`int(dt.timestamp() * 1000)` is the conversion). A future
  overload must be timezone-aware.
- The type stub and docs list the new keyword and property.

### Timezone
A fixed clock is an instant, not a serial. NOW shows it in the workbook's timezone
(`WorkbookOptions.timezone`, default `"UTC"`), or in the engine's `NOW(tz)` argument. So
`Clock::fixed_unix_millis(0)` gives `NOW()` = 25569 in UTC and 25568.791666… (19:00 the day
before) in `America/New_York` (UTC-5), and TODAY gives 25569 and 25568. The METADATA serial is read as UTC, as
it is today, so the harness, which runs in UTC, gives back exactly the cached serial. Changing the
timezone keeps the clock.

## Alternatives considered
- **Keep the thread-local and expose a public setter for it.** Rejected: it is per-thread, not
  per-workbook. It breaks for two workbooks on one thread and for a workbook moved across threads,
  and it leaks state between tests.
- **A fixed Excel serial instead of an instant.** It would be closer to what a spreadsheet user
  types. But it depends on the date system and the timezone, and the exporter and the year logic
  need an instant anyway. Rejected. Callers convert with `(serial - 25569) * 86_400_000` in UTC, for serials of 61 and above
  (below 61 the fictitious 1900-02-29 shifts the count by one day).
- **A `Fn() -> i64` callback.** It would allow simulated time, but it runs foreign code inside
  evaluation (directive 5), is not `Copy`, and complicates `Send` and the Python GIL. Rejected for
  v0.
- **`set_clock` outside the edit model (set the clock, no recalc).** Reads would show stale NOW
  values until the next edit, which breaks SPEC-0001's "last computed equals current" in v0.
  Rejected.
- **Keep the `mock_time` feature name.** Its name describes the mechanism this spec removes.
  Renamed to `metadata_clock`, which only gates the METADATA convention.

## Test plan
- **Engine (T-1479).**
  - Two models on one thread with fixed clocks of 0 ms and 86,400,000 ms give `NOW()` 25569 and
    25570 after `evaluate()`.
  - A model built without a clock reads the system time: NOW is within a few seconds of
    `SystemTime::now()`.
  - `fixed_unix_millis` accepts both bounds and rejects one millisecond beyond each.
  - At both bounds NOW and TODAY give numbers, with no error and no panic. `TODAY()` at the upper
    bound is 2958465 (9999-12-31).
  - In UTC, `TODAY()` is 1 at 1900-01-01T00:00Z, 59 at 1900-02-28 and 61 at 1900-03-01, matching
    `DATE()` for those dates.
  - Year-less `"7/1"+0` follows the fixed year: 2022 under the harness instant, 2030 under an
    instant in 2030.
  - The exporter's `dcterms:modified` equals the fixed instant, and two saves are byte-identical.
  - `rg get_milliseconds_since_epoch` and `rg MOCK_TIME` find nothing.
  - `system_unix_millis` gives 0 at the Unix epoch.
  - It gives a negative value for a time before 1970.
  - It clamps to each bound for `UNIX_EPOCH ± 400 years` and for `UNIX_EPOCH + Duration::MAX`
    when the platform can represent it, with no panic and no wrap.
  - `NOW()` and `TODAY()` at both fixed bounds in `Pacific/Kiritimati` (UTC+14) and
    `Pacific/Pago_Pago` (UTC-11) give a number or `#ERROR!`, never a panic. The test records which.
- **Harness (T-1479).**
  - `make eval` gives 0 fixes and 0 regressions in every suite.
  - The volatile and run-invariant counts in report.json are unchanged.
- **Facade (T-1480).**
  - `WorkbookOptions { clock: fixed(0) }` → `=NOW()` 25569 and `=TODAY()` 25569 right after
    `new` and after `open` of a saved copy (the load-time recalc).
  - `set_clock(fixed(86_400_000))` returns a `ChangeSet` listing both cells, and `A2=A1+1` too.
  - `set_clock` to the same fixed clock lists no NOW/TODAY value change.
  - An `edit` that sets the clock and then fails restores the old clock, and values are unchanged.
- **Python (T-1480).**
  - The same three cases through `rs.Workbook(clock=0)` and `wb.clock = 86_400_000`, plus the
    `ValueError` and `TypeError` cases, `clock=True` included.
  - `wb.clock = None` makes `wb.clock` return `None`.

## Rollout
1. **T-1479** (engine, M, opus):
   - adds `clock.rs` and the `Model` field, and routes every reader through the field;
   - removes the thread-local and the `mock_time` features;
   - renames the facade feature to `metadata_clock` (`pin_clock_from_metadata` on the engine
     clock);
   - moves the eval adapter onto `HARNESS_CLOCK_MS`;
   - adds `WorkbookOptions.clock` and the facade re-export of `Clock`/`ClockError`, which the
     adapter needs;
   - lists the new public `WorkbookOptions` field in its PR as an API change: struct literals
     need `..Default::default()`, as with `date_system` in SPEC-0005.

   Merge gate: 0 eval delta.
2. **T-1480** (api, S, sonnet):
   - adds `Workbook::clock`/`set_clock` and `Txn::set_clock`;
   - adds the Python keyword and property, the tests, and the stub;
   - marks SPEC-0001's clock lines implemented.

Until T-1479 merges, the thread-local stays the only mechanism and nothing in the public API
changes.
