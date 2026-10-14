use crate::formatter::dates::convert_to_serial_number;

#[derive(Clone)]
pub(crate) struct Tz(String);

mod js {
    use wasm_bindgen::prelude::*;

    // Weirdly enough Intl.supportedValuesOf('timeZone') doesn't include "UTC" or "GMT"
    // but Intl.DateTimeFormat does support them, so we add them manually.
    #[wasm_bindgen(inline_js = r#"
        const _fmtCache = new Map();
        function getFormatter(tz) {
            let f = _fmtCache.get(tz);
            if (!f) {
                f = new Intl.DateTimeFormat('en-US', {
                    timeZone: tz, year: 'numeric', month: '2-digit', day: '2-digit',
                    hour: '2-digit', minute: '2-digit', second: '2-digit', hourCycle: 'h23'
                });
                _fmtCache.set(tz, f);
            }
            return f;
        }
        export function ic_tz_validate(name) {
            try { getFormatter(name); return true; }
            catch(_) { return false; }
        }
        export function ic_tz_all() {
            try {
                const list = Intl.supportedValuesOf('timeZone');

                const zones = new Set(list);

                const extras = [ 'UTC', 'GMT' ];

                for (const tz of extras) {
                    zones.add(tz);
                }

                return Array.from(zones);
            } catch(e) {
                // no support
                console.log(e);
                return [ ];
            }
        }
        export function ic_tz_parts(ms, tz) {
            const p = {};
            for (const x of getFormatter(tz).formatToParts(new Date(ms))) p[x.type] = x.value | 0;
            return [p.year, p.month, p.day, p.hour, p.minute, p.second];
        }
    "#)]
    extern "C" {
        pub fn ic_tz_validate(name: &str) -> bool;
        pub fn ic_tz_all() -> js_sys::Array;
        pub fn ic_tz_parts(ms: f64, tz: &str) -> js_sys::Array;
    }
}

impl Tz {
    pub(crate) fn parse(s: &str) -> Result<Tz, String> {
        if js::ic_tz_validate(s) {
            Ok(Tz(s.to_string()))
        } else {
            Err(format!("Invalid timezone: {s}"))
        }
    }
}

pub fn get_all_timezone_names() -> Vec<String> {
    js::ic_tz_all()
        .iter()
        .filter_map(|v| v.as_string())
        .collect()
}

/// The Excel serial of the instant `ms` (ms since the Unix epoch) in `tz`.
pub(crate) fn excel_serial_for_now(tz: &Tz, ms: i64) -> Option<f64> {
    let parts = js::ic_tz_parts(ms as f64, &tz.0);
    if parts.length() != 6 {
        return None;
    }

    let get = |i: u32| parts.get(i).as_f64().unwrap_or(0.0) as i32;
    let (year, month, day) = (get(0), get(1), get(2));
    let (hour, minute, second) = (get(3), get(4), get(5));
    // Days are numbered as DATE numbers them, fictitious 1900-02-29 included
    // (docs/excel-semantics/dates.md "Serial 60 is the fictitious 1900-02-29").
    let date = chrono::NaiveDate::from_ymd_opt(
        year,
        u32::try_from(month).ok()?,
        u32::try_from(day).ok()?,
    )?;
    let secs = hour * 3600 + minute * 60 + second;

    Some(f64::from(convert_to_serial_number(date)) + f64::from(secs) / 86400.0)
}

/// The calendar year in `tz` of the instant `ms` (ms since the Unix epoch).
pub(crate) fn current_year(tz: &Tz, ms: i64) -> Option<i32> {
    let parts = js::ic_tz_parts(ms as f64, &tz.0);
    (parts.length() == 6).then(|| parts.get(0).as_f64().unwrap_or(0.0) as i32)
}
