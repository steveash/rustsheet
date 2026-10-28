//! The run's own record in report.json (SPEC-0022, T-0080): the commit and UTC date it ran at,
//! whether the tree had uncommitted changes (T-0082), and each suite under eval/suites it did not
//! run with the reason, so the progress dashboard can tell a complete run from a stale one. Also
//! the report's timing section. `date`, `commit` and `dirty` change from run to run, so no test or
//! golden file compares a whole report.json.

use std::fmt::Write as _;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use rustsheet_engine::formatter::dates::from_excel_date;
use serde::{Deserialize, Serialize};

use crate::report::RunReport;
use crate::suite::{FetchState, Suite};

/// Why an external suite that was never fetched did not run (main.rs prints the same words).
pub const NOT_FETCHED: &str = "not fetched";
/// Why a suite that a `--suite` selection left out did not run.
pub const NOT_SELECTED: &str = "not selected";

/// Excel's serial number of 1970-01-01, the Unix epoch.
const UNIX_EPOCH_SERIAL: i64 = 25_569;

/// When and at which commit the report was made, and what it left out.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunMeta {
    /// `git rev-parse --short=8 HEAD` at the repository root, or `None` when git fails.
    pub commit: Option<String>,
    /// Whether `git status --porcelain` at the repository root listed any tracked or untracked
    /// change (ignored files do not count), so the report is not exactly `commit`'s result;
    /// `None` when git fails, like `commit`.
    #[serde(default)]
    pub dirty: Option<bool>,
    /// The UTC date of the run, `YYYY-MM-DD`; empty in a report no run filled.
    pub date: String,
    /// Each suite under eval/suites that did not run, by name.
    pub skipped_suites: Vec<SkippedSuite>,
}

/// One suite the run did not score.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedSuite {
    /// The suite's manifest name.
    pub name: String,
    /// [`NOT_FETCHED`] or [`NOT_SELECTED`].
    pub reason: String,
}

impl RunMeta {
    /// The record of a run at `root` started at `now` that ran `chosen` of the suites `all`.
    #[must_use]
    pub fn new(root: &Path, now: SystemTime, all: &[Suite], chosen: &[Suite]) -> RunMeta {
        RunMeta {
            commit: git_commit(root),
            dirty: git_dirty(root),
            date: utc_date(now),
            skipped_suites: skipped(all, chosen),
        }
    }
}

/// The short commit of `root`'s `HEAD`; `None` when git is missing, fails or prints no hash. A
/// git failure is never an error: the report just names no commit.
#[must_use]
pub fn git_commit(root: &Path) -> Option<String> {
    let out = git(root, &["rev-parse", "--short=8", "HEAD"])?;
    let sha = String::from_utf8(out.stdout).ok()?;
    let sha = sha.trim();
    (out.status.success() && !sha.is_empty() && sha.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| sha.to_string())
}

/// Whether `root`'s work tree has uncommitted changes; `None` when git is missing or fails.
#[must_use]
pub fn git_dirty(root: &Path) -> Option<bool> {
    let out = git(root, &["status", "--porcelain"])?;
    out.status
        .success()
        .then(|| porcelain_is_dirty(&String::from_utf8_lossy(&out.stdout)))
}

/// Whether `git status --porcelain` output lists any entry: each changed or untracked path is
/// one non-empty line, and a clean tree prints nothing.
#[must_use]
pub fn porcelain_is_dirty(porcelain: &str) -> bool {
    porcelain.lines().any(|line| !line.trim().is_empty())
}

/// The output of `git -C root <args>`, run against `root`'s own repository whatever `GIT_DIR` or
/// `GIT_WORK_TREE` say; `None` when git cannot be started.
fn git(root: &Path, args: &[&str]) -> Option<Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()
}

/// `now`'s UTC date as `YYYY-MM-DD`. A clock before 1970 reads as 1970-01-01, and one past
/// Excel's calendar (9999-12-31) as an empty date.
#[must_use]
pub fn utc_date(now: SystemTime) -> String {
    let days = now
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86_400);
    i64::try_from(days)
        .ok()
        .and_then(|d| d.checked_add(UNIX_EPOCH_SERIAL))
        .and_then(|serial| from_excel_date(serial).ok())
        .map(|d| d.to_string())
        .unwrap_or_default()
}

/// The suites of `all` not in `chosen`, each with the reason it did not run.
#[must_use]
pub fn skipped(all: &[Suite], chosen: &[Suite]) -> Vec<SkippedSuite> {
    all.iter()
        .filter(|s| !chosen.iter().any(|c| c.manifest.name == s.manifest.name))
        .map(|s| SkippedSuite {
            name: s.manifest.name.clone(),
            reason: if s.fetch_state() == FetchState::NotFetched {
                NOT_FETCHED
            } else {
                NOT_SELECTED
            }
            .to_string(),
        })
        .collect()
}

/// The report's run line: commit, date and the skipped suites.
pub fn write_markdown(run: &RunMeta, md: &mut String) {
    let skipped: Vec<String> = run
        .skipped_suites
        .iter()
        .map(|s| format!("{} ({})", s.name, s.reason))
        .collect();
    let _ = writeln!(
        md,
        "Run: commit `{}`{}, {} (UTC). Skipped suites: {}.\n",
        run.commit.as_deref().unwrap_or("unknown"),
        if run.dirty == Some(true) {
            " with uncommitted changes"
        } else {
            ""
        },
        if run.date.is_empty() {
            "no date"
        } else {
            &run.date
        },
        if skipped.is_empty() {
            "none".to_string()
        } else {
            skipped.join(", ")
        }
    );
}

/// The timing section: total time per suite, then the slowest cases.
pub fn write_timing(report: &RunReport, md: &mut String) {
    let _ = writeln!(md, "\n## Timing\n");
    let _ = writeln!(md, "| suite | total ms |\n|---|---|");
    for s in &report.suites {
        let _ = writeln!(md, "| {} | {} |", s.name, s.millis);
    }
    let _ = writeln!(md, "\nSlowest {} cases:\n", report.slowest.len());
    let _ = writeln!(md, "| case | ms |\n|---|---|");
    for c in &report.slowest {
        let _ = writeln!(md, "| {} | {} |", c.id, c.millis);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suite::SuiteManifest;
    use std::path::PathBuf;
    use std::time::Duration;

    fn suite(name: &str, fetch: &str) -> Suite {
        let text = format!("name = \"{name}\"\nkind = \"xlsx\"\nlicense = \"MIT\"\n{fetch}");
        let manifest: SuiteManifest = toml::from_str(&text).expect("manifest parses");
        Suite {
            manifest,
            dir: PathBuf::from("eval/suites").join(name),
            // Never created, so an external suite reads as not fetched.
            cache_dir: std::env::temp_dir().join("rustsheet-run-meta-test-no-such-dir"),
        }
    }

    #[test]
    fn an_unfetched_suite_and_a_deselected_one_are_skipped_with_their_reason() {
        let fetch = "[fetch]\nurl = \"https://example.com/r.git\"\nrev = \"0123456789abcdef0123456789abcdef01234567\"\n";
        let all = [suite("poi", fetch), suite("probes", ""), suite("smoke", "")];
        let chosen = [suite("smoke", "")];
        let got = skipped(&all, &chosen);
        let want = |name: &str, reason: &str| SkippedSuite {
            name: name.into(),
            reason: reason.into(),
        };
        assert_eq!(
            got,
            [want("poi", NOT_FETCHED), want("probes", NOT_SELECTED)]
        );
        let mut md = String::new();
        let run = RunMeta {
            skipped_suites: got,
            ..RunMeta::default()
        };
        write_markdown(&run, &mut md);
        assert_eq!(
            md,
            "Run: commit `unknown`, no date (UTC). Skipped suites: poi (not fetched), probes (not selected).\n\n"
        );
    }

    #[test]
    fn a_failing_git_gives_a_null_commit() {
        let missing = std::env::temp_dir().join("rustsheet-run-meta-test-no-such-dir");
        assert_eq!(git_commit(&missing), None);
        let run = RunMeta::new(&missing, UNIX_EPOCH, &[], &[]);
        assert_eq!((run.commit.clone(), run.dirty), (None, None));
        let json = serde_json::to_value(&run).unwrap_or_default();
        assert_eq!(json["commit"], serde_json::Value::Null);
        assert_eq!(json["dirty"], serde_json::Value::Null);
        assert_eq!(json["date"], "1970-01-01");
        assert_eq!(json["skipped_suites"], serde_json::json!([]));
    }

    #[test]
    fn any_porcelain_entry_makes_the_tree_dirty() {
        assert!(!porcelain_is_dirty(""));
        assert!(!porcelain_is_dirty("\n"));
        assert!(porcelain_is_dirty(
            " M crates/rustsheet-eval/src/run_meta.rs\n"
        ));
        assert!(porcelain_is_dirty("?? scratch.txt\n"));
        let mut md = String::new();
        let run = RunMeta {
            commit: Some("0123abcd".into()),
            dirty: Some(true),
            date: "2026-10-27".into(),
            ..RunMeta::default()
        };
        write_markdown(&run, &mut md);
        assert!(
            md.starts_with("Run: commit `0123abcd` with uncommitted changes, 2026-10-27 (UTC)."),
            "{md}"
        );
        // A report read back from before T-0082 has no `dirty`.
        let old: RunMeta = serde_json::from_str(
            r#"{"commit":"0123abcd","date":"2026-10-27","skipped_suites":[]}"#,
        )
        .unwrap_or_default();
        assert_eq!((old.commit.as_deref(), old.dirty), (Some("0123abcd"), None));
    }

    #[test]
    fn the_date_is_the_utc_day_of_the_run() {
        let day = UNIX_EPOCH + Duration::from_secs(1_792_886_400);
        assert_eq!(utc_date(day), "2026-10-25");
        assert_eq!(utc_date(day + Duration::from_secs(86_399)), "2026-10-25");
        assert_eq!(utc_date(day + Duration::from_secs(86_400)), "2026-10-26");
        assert_eq!(utc_date(UNIX_EPOCH - Duration::from_secs(1)), "1970-01-01");
    }
}
