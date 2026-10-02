use crate::constants::EXCEL_DATE_BASE;

#[derive(Clone)]
pub(crate) struct Tz(chrono_tz::Tz);

impl Tz {
    pub(crate) fn parse(s: &str) -> Result<Tz, String> {
        s.parse::<chrono_tz::Tz>()
            .map(Tz)
            .map_err(|_| format!("Invalid timezone: {s}"))
    }
}

pub fn get_all_timezone_names() -> Vec<String> {
    chrono_tz::TZ_VARIANTS
        .iter()
        .map(|t| t.name().to_string())
        .collect()
}

pub(crate) fn excel_serial_for_now(tz: &Tz) -> Option<f64> {
    use chrono::{DateTime, Datelike, Timelike};
    let millis = crate::model::get_milliseconds_since_epoch();
    DateTime::from_timestamp(millis.div_euclid(1000), 0).map(|dt| {
        let local = dt.with_timezone(&tz.0);
        let days = local.num_days_from_ce() - EXCEL_DATE_BASE;
        // Excel's NOW() keeps sub-second precision (see the cached NOW cells in
        // eval/suites/ironcalc-calc DATE_AND_TIME/NOW_TODAY.xlsx), so the fraction carries milliseconds.
        let seconds =
            f64::from(local.num_seconds_from_midnight()) + millis.rem_euclid(1000) as f64 / 1000.0;
        let fraction = seconds / (60.0 * 60.0 * 24.0);
        days as f64 + fraction
    })
}
