//! Suite manifests (`eval/suites/<name>/suite.toml`) and case discovery.

use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use walkdir::WalkDir;

use crate::xlsx_snapshot::WorkbookSnapshot;

/// What kind of files a suite contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SuiteKind {
    /// `.xlsx` workbooks; expectations from a sidecar `.oracle.json` or the cached values.
    Xlsx,
    /// `.case.json` spec cases; expectations inline or from a sidecar `.oracle.json`.
    Spec,
}

/// Granularity of the no-regression baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BaselineMode {
    /// Every passing cell is recorded (small, curated suites).
    #[default]
    Cells,
    /// Only fully passing cases are recorded (large corpora).
    Cases,
    /// No baseline; informational suite.
    None,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteManifest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub kind: SuiteKind,
    /// Directory containing the cases, relative to the manifest's directory.
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    /// SPDX license of the case files (informational; see docs/research/test-suites.md).
    pub license: String,
    /// Where the cases come from (URL or "generated" / "hand").
    #[serde(default)]
    pub origin: String,
    /// Use xlsx cached values as expectations when no oracle sidecar exists.
    #[serde(default)]
    pub use_cached_values: bool,
    /// Only trust cached values written by Microsoft Excel.
    #[serde(default = "default_true")]
    pub require_excel_authored: bool,
    /// Part of the CI no-regression gate.
    #[serde(default)]
    pub gate: bool,
    #[serde(default)]
    pub baseline: BaselineMode,
    /// Milestone tag(s) this suite primarily measures.
    #[serde(default)]
    pub milestone: Vec<String>,
    /// Directory (relative to the manifest) holding oracle sidecars in a tree mirroring the cases:
    /// `<oracle_dir>/<case path relative to the suite root, extension → .oracle.json>`. Needed when
    /// the cases live outside the repository (fetched suites) so the sidecars can be committed.
    /// When absent, a sidecar sits next to its case (`foo.xlsx` → `foo.oracle.json`).
    #[serde(default)]
    pub oracle_dir: Option<String>,
    /// xlsx suites: file (relative to the manifest) of per-cell replacements for cached values
    /// that are not en-US evidence (`overrides.rs`, decision 0032).
    #[serde(default)]
    pub cached_overrides: Option<String>,
    /// File (relative to the manifest) of cells that differ from Excel by design, scored apart
    /// from bit-exact (`deviations.rs`, decision 0036).
    #[serde(default)]
    pub deviations: Option<String>,
    /// Pinned external source; when present the cases live in `eval/.cache/suites/<name>`
    /// (populated by `rustsheet-eval fetch`) instead of next to the manifest.
    #[serde(default)]
    pub fetch: Option<FetchSpec>,
}

/// Pinned download of a suite too large or not redistributable to vendor.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchSpec {
    /// `https://` or `file://` URL of a git repository (with `rev`) or an archive (with `sha256`).
    pub url: String,
    /// Full 40-hex commit id; makes the source a git repository.
    #[serde(default)]
    pub rev: Option<String>,
    /// SHA-256 (hex) of the archive; makes the source a `.zip`/`.tar*` archive.
    #[serde(default)]
    pub sha256: Option<String>,
    /// Directory inside the source that becomes the suite root.
    #[serde(default)]
    pub subdir: Option<String>,
    /// Globs (relative to `subdir`) of the files to keep; default everything.
    #[serde(default = "default_include")]
    pub include: Vec<String>,
}

impl FetchSpec {
    /// The pin that a fetched copy must carry to count as up to date.
    #[must_use]
    pub fn pin(&self) -> String {
        let id = self
            .rev
            .as_deref()
            .or(self.sha256.as_deref())
            .unwrap_or_default();
        format!(
            "{}@{id}#{}",
            self.url,
            self.subdir.as_deref().unwrap_or_default()
        )
    }

    /// # Errors
    /// Fails unless the spec is a well-formed, fully pinned source.
    pub fn validate(&self) -> Result<()> {
        if !(self.url.starts_with("https://") || self.url.starts_with("file://")) {
            bail!("fetch.url must start with https:// or file://");
        }
        let hex = |s: &str, len: usize| s.len() == len && s.bytes().all(|b| b.is_ascii_hexdigit());
        match (&self.rev, &self.sha256) {
            (Some(rev), None) if hex(rev, 40) => {}
            (None, Some(sha)) if hex(sha, 64) => {}
            _ => bail!(
                "fetch needs exactly one pin: a 40-hex `rev` (git) or a 64-hex `sha256` (archive)"
            ),
        }
        if let Some(sub) = &self.subdir {
            let unsafe_path = Path::new(sub).components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            });
            if unsafe_path {
                bail!("fetch.subdir must be a relative path without `..`");
            }
        }
        Ok(())
    }
}

/// Whether a suite's cases are available locally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchState {
    /// Vendored next to the manifest.
    Local,
    /// External and present at the pinned revision.
    Fetched,
    /// External and missing or stale; run `rustsheet-eval fetch`.
    NotFetched,
}

/// The name of Gate A's reference-table suite, resolved by [`Suite::load_named`] (SPEC-0029).
pub const ACCURACY_SUITE: &str = "accuracy";

/// Its manifest, relative to the repository root: outside `eval/suites/`, so `make eval`, the
/// eval gate and every report measure never see the tables (SPEC-0029).
pub const ACCURACY_MANIFEST: &str = "eval/accuracy/suite.toml";

/// File inside a fetched suite recording the pin it was fetched at.
pub const FETCH_MARKER: &str = ".rustsheet-fetched";

fn default_root() -> String {
    "cases".into()
}
fn default_include() -> Vec<String> {
    vec!["**/*".into()]
}
fn default_true() -> bool {
    true
}

impl SuiteManifest {
    /// Whether an xlsx case without an oracle sidecar is scored against its own cached values.
    /// The runner and `rustsheet-eval cases` (used by the oracle fill) share this one rule.
    #[must_use]
    pub fn trusts_cached_values(&self, snapshot: &WorkbookSnapshot) -> bool {
        self.use_cached_values && (!self.require_excel_authored || snapshot.is_excel_authored())
    }
}

#[derive(Debug, Clone)]
pub struct Suite {
    pub manifest: SuiteManifest,
    pub dir: PathBuf,
    /// Where an external suite is fetched to (`eval/.cache/suites/<name>`).
    pub cache_dir: PathBuf,
}

/// A discovered case file.
#[derive(Debug, Clone)]
pub struct CaseFile {
    /// `<suite>/<path relative to suite root>`.
    pub id: String,
    pub path: PathBuf,
    /// Where this case's oracle sidecar lives (see `SuiteManifest::oracle_dir`).
    pub sidecar: PathBuf,
}

pub(crate) fn build_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        b.add(Glob::new(p).with_context(|| format!("bad glob {p}"))?);
    }
    Ok(b.build()?)
}

impl Suite {
    /// Loads every `suite.toml` under `suites_dir`.
    ///
    /// # Errors
    /// Fails if a manifest cannot be read or parsed.
    pub fn load_all(suites_dir: &Path) -> Result<Vec<Suite>> {
        let mut suites = Vec::new();
        for entry in WalkDir::new(suites_dir).min_depth(2).max_depth(2) {
            let entry = entry?;
            if entry.file_name() == "suite.toml" {
                suites.push(Self::load_manifest(entry.path(), suites_dir)?);
            }
        }
        suites.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
        Ok(suites)
    }

    /// The suite called `name`: one of `eval/suites/` under `root`, or Gate A's reference tables
    /// (`eval/accuracy/suite.toml`) for [`ACCURACY_SUITE`], which no measure of `load_all` sees
    /// (SPEC-0029). A suite under `eval/suites/` may not take that name, so it is never ambiguous.
    ///
    /// # Errors
    /// When no suite has the name, a manifest is unreadable, or `eval/suites/` holds a suite
    /// named [`ACCURACY_SUITE`].
    pub fn load_named(root: &Path, name: &str) -> Result<Suite> {
        let suites_dir = root.join("eval/suites");
        let mut all = if suites_dir.is_dir() {
            Self::load_all(&suites_dir)?
        } else {
            Vec::new()
        };
        if let Some(i) = all.iter().position(|s| s.manifest.name == name) {
            if name == ACCURACY_SUITE {
                bail!(
                    "{}: the suite name {ACCURACY_SUITE} is reserved for eval/accuracy (SPEC-0029)",
                    all[i].dir.display()
                );
            }
            return Ok(all.swap_remove(i));
        }
        if name == ACCURACY_SUITE {
            let suite = Self::load_manifest(&root.join(ACCURACY_MANIFEST), &suites_dir)?;
            if suite.manifest.name != ACCURACY_SUITE {
                bail!("{ACCURACY_MANIFEST}: the suite must be named {ACCURACY_SUITE}");
            }
            return Ok(suite);
        }
        bail!("suite {name} not found")
    }

    /// Reads and validates the manifest at `path`; `suites_dir` places a fetched suite's cache.
    fn load_manifest(path: &Path, suites_dir: &Path) -> Result<Suite> {
        let text = std::fs::read_to_string(path).with_context(|| path.display().to_string())?;
        let manifest: SuiteManifest =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        // The name becomes a cache directory that `fetch` replaces wholesale.
        if manifest.name.is_empty()
            || !manifest
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            bail!("{}: suite name must match [A-Za-z0-9_-]+", path.display());
        }
        if let Some(d) = &manifest.oracle_dir {
            if d.is_empty() || Path::new(d).is_absolute() || d.split('/').any(|c| c == "..") {
                bail!(
                    "{}: oracle_dir must be a relative path inside the suite directory",
                    path.display()
                );
            }
        }
        if let Some(f) = &manifest.fetch {
            f.validate()
                .with_context(|| format!("{}: [fetch]", path.display()))?;
        }
        let cache_dir = suites_dir.join("../.cache/suites").join(&manifest.name);
        Ok(Suite {
            manifest,
            dir,
            cache_dir,
        })
    }

    #[must_use]
    pub fn root(&self) -> PathBuf {
        if self.manifest.fetch.is_some() {
            return self.cache_dir.clone();
        }
        self.dir.join(&self.manifest.root)
    }

    #[must_use]
    pub fn fetch_state(&self) -> FetchState {
        let Some(spec) = &self.manifest.fetch else {
            return FetchState::Local;
        };
        match std::fs::read_to_string(self.cache_dir.join(FETCH_MARKER)) {
            Ok(m) if m.trim() == spec.pin() => FetchState::Fetched,
            _ => FetchState::NotFetched,
        }
    }

    /// Lists the case files of this suite in a stable order.
    ///
    /// # Errors
    /// Fails on invalid glob patterns or unreadable directories.
    pub fn cases(&self) -> Result<Vec<CaseFile>> {
        let include = build_globs(&self.manifest.include)?;
        let exclude = build_globs(&self.manifest.exclude)?;
        let root = self.root();
        if !root.exists() {
            return Ok(Vec::new()); // fetched suite not downloaded yet
        }
        let ext = match self.manifest.kind {
            SuiteKind::Xlsx => ".xlsx",
            SuiteKind::Spec => ".case.json",
        };
        let mut out = Vec::new();
        for entry in WalkDir::new(&root).sort_by_file_name() {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(&root)?
                .to_string_lossy()
                .replace('\\', "/");
            let name = entry.file_name().to_string_lossy();
            if !name.ends_with(ext) || name.starts_with("~$") {
                continue;
            }
            if include.is_match(&rel) && !exclude.is_match(&rel) {
                let sidecar = match &self.manifest.oracle_dir {
                    Some(d) => oracle_sidecar(&self.dir.join(d).join(&rel)),
                    None => oracle_sidecar(entry.path()),
                };
                out.push(CaseFile {
                    id: format!("{}/{rel}", self.manifest.name),
                    path: entry.path().to_path_buf(),
                    sidecar,
                });
            }
        }
        Ok(out)
    }
}

/// The sidecar path for a case path: same directory, `foo.xlsx` / `foo.xlsm` / `foo.case.json`
/// → `foo.oracle.json`. With `oracle_dir`, `path` is the case's mirror path under that directory.
#[must_use]
pub fn oracle_sidecar(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let stem = name
        .strip_suffix(".case.json")
        .or_else(|| name.strip_suffix(".xlsx"))
        .or_else(|| name.strip_suffix(".xlsm"))
        .unwrap_or(&name);
    path.with_file_name(format!("{stem}.oracle.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REV: &str = "0123456789abcdef0123456789abcdef01234567";

    fn manifest(fetch: &str) -> SuiteManifest {
        let text = format!("name = \"ext\"\nkind = \"xlsx\"\nlicense = \"GPL-3.0\"\n{fetch}");
        toml::from_str(&text).expect("manifest parses")
    }

    fn suite_with(fetch: &str, cache_dir: PathBuf) -> Suite {
        Suite {
            manifest: manifest(fetch),
            dir: PathBuf::from("eval/suites/ext"),
            cache_dir,
        }
    }

    #[test]
    fn fetch_table_parses_and_validates() {
        let m = manifest(&format!(
            "[fetch]\nurl = \"https://example.com/r.git\"\nrev = \"{REV}\"\nsubdir = \"a/b\"\ninclude = [\"**/*.xlsx\"]\n"
        ));
        let f = m.fetch.expect("fetch table");
        assert_eq!(f.subdir.as_deref(), Some("a/b"));
        assert_eq!(f.include, vec!["**/*.xlsx".to_string()]);
        assert!(f.validate().is_ok());
    }

    #[test]
    fn fetch_requires_exactly_one_well_formed_pin() {
        let spec = |extra: &str| {
            manifest(&format!("[fetch]\nurl = \"https://x.org/a\"\n{extra}"))
                .fetch
                .expect("fetch table")
        };
        assert!(spec("").validate().is_err());
        assert!(spec("rev = \"main\"").validate().is_err());
        assert!(spec(&format!("rev = \"{REV}\"\nsha256 = \"{REV}\""))
            .validate()
            .is_err());
        assert!(spec(&format!("sha256 = \"{}\"", "a".repeat(64)))
            .validate()
            .is_ok());
    }

    #[test]
    fn fetch_rejects_unsafe_url_and_subdir() {
        let spec = |url: &str, sub: &str| FetchSpec {
            url: url.into(),
            rev: Some(REV.into()),
            sha256: None,
            subdir: Some(sub.into()),
            include: default_include(),
        };
        assert!(spec("http://x.org/a", "ok").validate().is_err());
        assert!(spec("-oProxy=x", "ok").validate().is_err());
        assert!(spec("https://x.org/a", "../up").validate().is_err());
        assert!(spec("https://x.org/a", "/abs").validate().is_err());
    }

    #[test]
    fn unknown_fetch_keys_are_rejected() {
        let text = "name = \"x\"\nkind = \"xlsx\"\nlicense = \"MIT\"\n[fetch]\nurl = \"https://x\"\nbogus = 1\n";
        assert!(toml::from_str::<SuiteManifest>(text).is_err());
    }

    #[test]
    fn external_suite_root_is_the_cache_and_unfetched_has_no_cases() {
        let cache = std::env::temp_dir().join(format!("rs-suite-test-{}", std::process::id()));
        let s = suite_with(
            &format!("[fetch]\nurl = \"https://x.org/a\"\nrev = \"{REV}\"\n"),
            cache.clone(),
        );
        assert_eq!(s.root(), cache);
        assert_eq!(s.fetch_state(), FetchState::NotFetched);
        assert!(s.cases().expect("cases").is_empty());

        std::fs::create_dir_all(&cache).expect("mkdir");
        std::fs::write(
            cache.join(FETCH_MARKER),
            s.manifest.fetch.as_ref().expect("fetch").pin(),
        )
        .expect("marker");
        assert_eq!(s.fetch_state(), FetchState::Fetched);
        std::fs::write(cache.join(FETCH_MARKER), "stale pin").expect("marker");
        assert_eq!(s.fetch_state(), FetchState::NotFetched);
        std::fs::remove_dir_all(&cache).expect("cleanup");
    }

    #[test]
    fn oracle_dir_mirrors_the_case_tree_under_the_suite_dir() {
        let base = std::env::temp_dir().join(format!("rs-suite-oracle-{}", std::process::id()));
        let cache = base.join("cache");
        std::fs::create_dir_all(cache.join("sub")).expect("mkdir");
        std::fs::write(cache.join("sub/b.xlsx"), b"").expect("case");
        let mut s = suite_with(
            &format!(
                "oracle_dir = \"oracle\"\n[fetch]\nurl = \"https://x.org/a\"\nrev = \"{REV}\"\n"
            ),
            cache.clone(),
        );
        s.dir = base.join("suite");
        let cases = s.cases().expect("cases");
        std::fs::remove_dir_all(&base).expect("cleanup");
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].id, "ext/sub/b.xlsx");
        assert_eq!(
            cases[0].sidecar,
            base.join("suite/oracle/sub/b.oracle.json")
        );
    }

    #[test]
    fn oracle_dir_must_stay_inside_the_suite() {
        let base = std::env::temp_dir().join(format!("rs-suite-bad-oracle-{}", std::process::id()));
        for (i, bad) in ["../x", "/abs", ""].iter().enumerate() {
            let dir = base.join(format!("s{i}"));
            std::fs::create_dir_all(dir.join("s")).expect("mkdir");
            std::fs::write(
                dir.join("s/suite.toml"),
                format!(
                    "name = \"s\"\nkind = \"spec\"\nlicense = \"MIT\"\noracle_dir = \"{bad}\"\n"
                ),
            )
            .expect("manifest");
            assert!(Suite::load_all(&dir).is_err(), "{bad:?} accepted");
        }
        std::fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn sidecar_name_strips_every_case_extension() {
        for case in ["a.xlsx", "a.xlsm", "a.case.json"] {
            assert_eq!(
                oracle_sidecar(Path::new(case)),
                PathBuf::from("a.oracle.json")
            );
        }
    }

    fn write_manifest(dir: &Path, name: &str) {
        std::fs::create_dir_all(dir).expect("mkdir");
        std::fs::write(
            dir.join("suite.toml"),
            format!("name = \"{name}\"\nkind = \"spec\"\nlicense = \"MIT\"\nroot = \"tables\"\n"),
        )
        .expect("manifest");
    }

    #[test]
    fn load_named_finds_eval_suites_and_the_accuracy_tables() {
        let root = std::env::temp_dir().join(format!("rs-suite-named-{}", std::process::id()));
        write_manifest(&root.join("eval/suites/smoke"), "smoke");
        write_manifest(&root.join("eval/accuracy"), ACCURACY_SUITE);
        let smoke = Suite::load_named(&root, "smoke").map(|s| s.dir);
        let accuracy = Suite::load_named(&root, ACCURACY_SUITE).map(|s| s.root());
        let missing = Suite::load_named(&root, "nope");
        // `load_all` never sees the tables.
        let all = Suite::load_all(&root.join("eval/suites")).map(|v| v.len());
        // A suite under eval/suites may not take the reserved name.
        write_manifest(&root.join("eval/suites/acc"), ACCURACY_SUITE);
        let ambiguous = Suite::load_named(&root, ACCURACY_SUITE);
        std::fs::remove_dir_all(&root).expect("cleanup");
        assert_eq!(smoke.expect("smoke"), root.join("eval/suites/smoke"));
        assert_eq!(
            accuracy.expect("accuracy"),
            root.join("eval/accuracy/tables")
        );
        assert!(missing.is_err());
        assert_eq!(all.expect("all"), 1);
        assert!(ambiguous.is_err());
    }

    #[test]
    fn the_accuracy_manifest_must_be_named_accuracy() {
        let root = std::env::temp_dir().join(format!("rs-suite-acc-name-{}", std::process::id()));
        write_manifest(&root.join("eval/accuracy"), "other");
        let got = Suite::load_named(&root, ACCURACY_SUITE);
        std::fs::remove_dir_all(&root).expect("cleanup");
        assert!(got.is_err());
    }

    #[test]
    fn local_suite_is_local() {
        let s = suite_with("", PathBuf::new());
        assert_eq!(s.fetch_state(), FetchState::Local);
        assert_eq!(s.root(), PathBuf::from("eval/suites/ext/cases"));
    }
}
