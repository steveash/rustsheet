//! The harness's view of the function catalog: which Excel functions are in scope for parity
//! (decision 0027) and which the excel-graph oracle cannot evaluate.
//!
//! The source of truth is `eval/catalog/functions.yaml`; tools/ generates the JSON view read
//! here (`cd tools && uv run rustsheet-catalog-view`), and `tools/tests/test_catalog_view.py`
//! fails when the two drift apart (decision 0040).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Path of the generated catalog view, relative to the repository root.
pub const VIEW_PATH: &str = "eval/catalog/scope.json";

/// Whether a function counts towards parity (decision 0027).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// Counted: the result depends only on the workbook.
    In,
    /// Not counted: the result depends on systems outside the workbook.
    Out,
}

/// What the excel-graph oracle can do for a function, when the catalog says anything (absent =
/// it evaluates the function). One variant today; the catalog may grow others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum OracleSupport {
    /// `oracle: none`: the oracle cannot return any value (INFO, SHEETS), so evidence must come
    /// from elsewhere.
    #[serde(rename = "none")]
    Unavailable,
}

/// One catalog entry, as far as the harness needs it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogFunction {
    /// Upper-case Excel name, e.g. `XLOOKUP`.
    pub name: String,
    /// Functional family (`math`, `text`, `lookup`, ...), which groups the evidence-floor list
    /// (T-0056).
    pub category: String,
    /// First milestone in which the function is in scope for the gate.
    pub milestone: String,
    /// Whether the function counts towards parity.
    pub scope: Scope,
    /// Why an out-of-scope function is out.
    #[serde(default)]
    pub reason: Option<String>,
    /// Set when the oracle cannot evaluate the function.
    #[serde(default)]
    pub oracle: Option<OracleSupport>,
}

#[derive(Deserialize)]
struct View {
    functions: Vec<CatalogFunction>,
}

/// Failure to read the catalog view.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    /// The view file could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The file that failed.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
    /// The view is not valid JSON of the expected shape.
    #[error("parsing the catalog view: {0} (regenerate with `cd tools && uv run rustsheet-catalog-view`)")]
    Parse(#[from] serde_json::Error),
}

/// Every catalog function by name. An empty catalog puts every function in scope.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    functions: BTreeMap<String, CatalogFunction>,
}

impl Catalog {
    /// Reads [`VIEW_PATH`] under the repository root `root`.
    ///
    /// # Errors
    /// When the file is missing or malformed.
    pub fn load(root: &Path) -> Result<Catalog, CatalogError> {
        let path = root.join(VIEW_PATH);
        let text = std::fs::read_to_string(&path).map_err(|source| CatalogError::Io {
            path: path.clone(),
            source,
        })?;
        Catalog::parse(&text)
    }

    /// Parses the JSON view.
    ///
    /// # Errors
    /// When `text` is not a catalog view.
    pub fn parse(text: &str) -> Result<Catalog, CatalogError> {
        let view: View = serde_json::from_str(text)?;
        Ok(Catalog::from_functions(view.functions))
    }

    /// A catalog of exactly `functions`.
    #[must_use]
    pub fn from_functions(functions: impl IntoIterator<Item = CatalogFunction>) -> Catalog {
        Catalog {
            functions: functions.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }

    /// All entries, by name.
    pub fn functions(&self) -> impl Iterator<Item = &CatalogFunction> {
        self.functions.values()
    }

    /// Whether `name` counts towards parity. Names the catalog does not list (user-defined or
    /// misparsed calls) count, so nothing drops out of the report silently.
    #[must_use]
    pub fn in_scope(&self, name: &str) -> bool {
        self.functions
            .get(name)
            .is_none_or(|f| f.scope == Scope::In)
    }

    /// Whether `name` is an in-scope function the oracle cannot evaluate.
    #[must_use]
    pub fn needs_other_evidence(&self, name: &str) -> bool {
        self.functions
            .get(name)
            .is_some_and(|f| f.scope == Scope::In && f.oracle == Some(OracleSupport::Unavailable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn committed_view_loads_and_marks_scope_and_oracle() {
        let catalog = Catalog::load(&repo_root()).unwrap_or_else(|e| panic!("{e}"));
        // T-0040: the catalog lists every Microsoft 365 Excel function.
        assert!(catalog.functions().count() >= 500);
        assert!(catalog.in_scope("SUM") && !catalog.needs_other_evidence("SUM"));
        assert!(!catalog.in_scope("WEBSERVICE"));
        assert!(catalog.needs_other_evidence("INFO"));
        assert!(catalog
            .functions()
            .filter(|f| f.scope == Scope::Out)
            .all(|f| f.reason.is_some()));
    }

    #[test]
    fn unlisted_names_are_in_scope() {
        assert!(Catalog::default().in_scope("MYUDF"));
        assert!(!Catalog::default().needs_other_evidence("MYUDF"));
    }

    #[test]
    fn unknown_fields_and_values_are_rejected() {
        let bad_scope = r#"{"functions": [{"name": "A", "category": "math", "milestone": "M2", "scope": "maybe"}]}"#;
        assert!(Catalog::parse(bad_scope).is_err());
        let extra = r#"{"functions": [{"name": "A", "category": "math", "milestone": "M2", "scope": "in", "x": 1}]}"#;
        assert!(Catalog::parse(extra).is_err());
    }
}
