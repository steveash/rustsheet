//! Opt-in log of which cases are running, for post-mortems of a run that dies from a signal
//! (T-1118). Worker thread names (`case:<id>`) are cut to 15 bytes by Linux, so a debugger
//! backtrace cannot name the case; this log can. `scripts/eval_guard.sh` sets
//! [`CASE_LOG_ENV`] and, if the process is killed, lists the cases that started but never ended.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// Environment variable naming the file the log is appended to. Unset: no log.
pub const CASE_LOG_ENV: &str = "RUSTSHEET_EVAL_CASE_LOG";

static LOG: OnceLock<Option<Mutex<File>>> = OnceLock::new();

fn log_file() -> Option<&'static Mutex<File>> {
    LOG.get_or_init(|| {
        let path = std::env::var_os(CASE_LOG_ENV)?;
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
            .map(Mutex::new)
    })
    .as_ref()
}

/// Appends `<event>\t<case id>` when the log is enabled (case ids already start with their
/// suite). Best effort: a write error must never change a run's result, so it is ignored.
pub fn record(event: &str, case_id: &str) {
    if let Some(file) = log_file() {
        if let Ok(mut file) = file.lock() {
            // One `write_all` per line, so a crash leaves at most the last line torn.
            let _ = file.write_all(line(event, case_id).as_bytes());
        }
    }
}

/// One log line. Control characters in the id become spaces, so an id cannot forge a line.
fn line(event: &str, case_id: &str) -> String {
    let id: String = case_id
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    format!("{event}\t{id}\n")
}

#[cfg(test)]
mod tests {
    use super::line;

    #[test]
    fn line_is_event_tab_id() {
        // scripts/eval_guard.sh parses exactly this shape.
        assert_eq!(
            line("start", "probes/a.case.json"),
            "start\tprobes/a.case.json\n"
        );
        assert_eq!(line("end", "s/x\tend\ny"), "end\ts/x end y\n");
    }
}
