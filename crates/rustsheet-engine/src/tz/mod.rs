#[cfg(target_arch = "wasm32")]
mod browser_tz;
#[cfg(target_arch = "wasm32")]
pub(crate) use browser_tz::*;

#[cfg(not(target_arch = "wasm32"))]
mod os_tz;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use os_tz::*;

impl crate::model::Model {
    /// The current year in the model's timezone (the clock NOW and TODAY read), which year-less
    /// date text ("7/1", "Jul-26") is read in (docs/excel-semantics/dates.md, "Year-less text").
    pub(crate) fn current_year(&self) -> Option<i32> {
        current_year(&self.tz, self.now_unix_millis())
    }
}
