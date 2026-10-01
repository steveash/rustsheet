//! Suite manifests (`eval/suites/<name>/suite.toml`) and case discovery.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use walkdir::WalkDir;

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
}

fn default_root() -> String {
    "cases".into()
}
fn default_include() -> Vec<String> {
    vec!["**/*".into()]
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct Suite {
    pub manifest: SuiteManifest,
    pub dir: PathBuf,
}

/// A discovered case file.
#[derive(Debug, Clone)]
pub struct CaseFile {
    /// `<suite>/<path relative to suite root>`.
    pub id: String,
    pub path: PathBuf,
}

fn build_globs(patterns: &[String]) -> Result<GlobSet> {
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
                let text = std::fs::read_to_string(entry.path())?;
                let manifest: SuiteManifest = toml::from_str(&text)
                    .with_context(|| format!("parsing {}", entry.path().display()))?;
                let dir = entry
                    .path()
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_default();
                suites.push(Suite { manifest, dir });
            }
        }
        suites.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
        Ok(suites)
    }

    #[must_use]
    pub fn root(&self) -> PathBuf {
        self.dir.join(&self.manifest.root)
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
                out.push(CaseFile {
                    id: format!("{}/{rel}", self.manifest.name),
                    path: entry.path().to_path_buf(),
                });
            }
        }
        Ok(out)
    }
}

/// Path of the oracle sidecar for a case file (`foo.xlsx` → `foo.oracle.json`).
#[must_use]
pub fn oracle_sidecar(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let stem = name
        .strip_suffix(".case.json")
        .or_else(|| name.strip_suffix(".xlsx"))
        .unwrap_or(&name);
    path.with_file_name(format!("{stem}.oracle.json"))
}
