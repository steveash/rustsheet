#[cfg(target_arch = "wasm32")]
mod browser_tz;
#[cfg(target_arch = "wasm32")]
pub(crate) use browser_tz::*;

#[cfg(not(target_arch = "wasm32"))]
mod os_tz;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use os_tz::*;

/// The calendar year of "now" in `tz`, the clock NOW and TODAY read: year-less date text
/// ("7/1", "Jul-26") is read in it (docs/excel-semantics/dates.md, "Year-less text").
pub(crate) fn current_year(tz: &Tz) -> Option<i32> {
    let serial = excel_serial_for_now(tz)?.floor();
    // Serials are far inside i64; `excel_serial_to_ymd` rejects any out of Excel's range.
    #[allow(clippy::cast_possible_truncation)]
    let days = serial as i64;
    crate::formatter::dates::excel_serial_to_ymd(days).map(|(year, _, _)| year)
}
