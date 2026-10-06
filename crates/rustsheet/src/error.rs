use crate::refs::CellRef;

/// A reference (sheet, cell or range) could not be resolved.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum RefError {
    /// No sheet with this name or id.
    #[error("unknown sheet {0:?}")]
    UnknownSheet(String),
    /// The text is not a valid reference.
    #[error("invalid reference {0:?}")]
    Invalid(String),
    /// Row or column outside the Excel limits (1,048,576 x 16,384).
    #[error("reference out of bounds")]
    OutOfBounds,
    /// The range exceeds the configured read limit.
    #[error("range too large")]
    TooLarge,
    /// A reference form v0 does not accept (defined names, structured references).
    #[error("unsupported reference form")]
    Unsupported,
}

impl RefError {
    /// Classifies an engine `String` error from a sheet, row or column lookup.
    ///
    /// `subject` is what the caller looked up (a sheet name or reference text); the engine's
    /// message does not carry it. Returns `None` for messages that are not reference errors, so
    /// the caller can fall back to an `Engine` catch-all variant.
    #[must_use]
    pub fn from_engine_message(message: &str, subject: &str) -> Option<Self> {
        // Messages produced by `Workbook::worksheet*`, `Model::cell_reference_to_string`, link parsing
        // and the frozen-pane accessors; `tests::engine_messages_are_classified` triggers the real
        // engine paths so a reworded message fails there.
        match message {
            "Invalid sheet" | "Invalid sheet index" | "Scope: Invalid sheet index" => {
                Some(Self::UnknownSheet(subject.to_string()))
            }
            "Too many rows" | "Too many columns" => Some(Self::OutOfBounds),
            m if m.starts_with("Invalid row") || m.starts_with("Invalid column") => {
                Some(Self::OutOfBounds)
            }
            m if m.starts_with("Error parsing reference") => {
                Some(Self::Invalid(subject.to_string()))
            }
            _ => None,
        }
    }
}

/// An edit was rejected; the workbook is left unchanged.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum EditError {
    /// The target reference was invalid.
    #[error(transparent)]
    Ref(#[from] RefError),
    /// The input could not be applied to the cell (e.g. an unparseable formula).
    #[error("invalid input for {cell}: {message}")]
    InvalidInput {
        /// The cell being written.
        cell: crate::refs::CellRef,
        /// What the engine said.
        message: String,
    },
    /// The sheet name is empty, too long, duplicated or contains forbidden characters.
    #[error("invalid sheet name: {0}")]
    SheetName(String),
    /// A workbook must keep at least one sheet.
    #[error("cannot delete the last sheet")]
    LastSheet,
    /// The defined name is invalid or already exists, or its formula does not parse.
    #[error("invalid defined name: {0}")]
    DefinedName(String),
    /// Any other engine error, passed through verbatim.
    #[error("engine error: {0}")]
    Engine(String),
}

impl EditError {
    /// Classifies an engine `String` error from a write: sheet-name and last-sheet failures, then
    /// reference errors (when `cell` is the write's target), else the catch-all variants.
    ///
    /// `tests::edit_messages_are_classified` triggers the real engine paths so a reworded
    /// message fails there.
    pub(crate) fn from_engine_message(message: String, cell: Option<CellRef>) -> Self {
        if message.starts_with("Invalid name for a sheet")
            || message.contains("already exists")
            || message.starts_with("Sheet already exists")
        {
            return Self::SheetName(message);
        }
        if message.starts_with("Cannot delete only sheet") {
            return Self::LastSheet;
        }
        let Some(cell) = cell else {
            return Self::Engine(message);
        };
        match RefError::from_engine_message(&message, &cell.to_string()) {
            Some(e) => Self::Ref(e),
            None => Self::InvalidInput { cell, message },
        }
    }
}

/// A workbook could not be loaded.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum LoadError {
    /// Reading the file failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The file is not a valid workbook.
    #[error("invalid workbook: {0}")]
    Format(String),
    /// The load options are inconsistent.
    #[error("invalid options: {0}")]
    Options(String),
}

/// A workbook could not be saved.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum SaveError {
    /// Writing the file failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Any other engine error, passed through verbatim.
    #[error("engine error: {0}")]
    Engine(String),
}

#[cfg(test)]
mod tests {
    use rustsheet_engine::Model;

    use super::*;
    use crate::refs::SheetId;

    #[test]
    fn engine_messages_are_classified() {
        let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
        let sheet = model.workbook.worksheet(10).map(|_| ()).unwrap_err();
        assert!(matches!(
            RefError::from_engine_message(&sheet, "Nope"),
            Some(RefError::UnknownSheet(s)) if s == "Nope"
        ));
        let pane = model.get_frozen_rows_count(10).unwrap_err();
        assert!(matches!(
            RefError::from_engine_message(&pane, "10"),
            Some(RefError::UnknownSheet(_))
        ));
        let rows = model.set_frozen_rows(0, 2_000_000).unwrap_err();
        assert!(matches!(
            RefError::from_engine_message(&rows, "2000000"),
            Some(RefError::OutOfBounds)
        ));
        let columns = model.set_frozen_columns(0, 20_000).unwrap_err();
        assert!(matches!(
            RefError::from_engine_message(&columns, "20000"),
            Some(RefError::OutOfBounds)
        ));
        for message in ["Invalid row", "Invalid column: 'XFE'"] {
            assert!(matches!(
                RefError::from_engine_message(message, "x"),
                Some(RefError::OutOfBounds)
            ));
        }
        assert!(matches!(
            RefError::from_engine_message("Error parsing reference: 'A$'", "A$"),
            Some(RefError::Invalid(s)) if s == "A$"
        ));
        assert!(RefError::from_engine_message("Cannot edit a cell", "A1").is_none());
    }

    #[test]
    fn edit_messages_are_classified() {
        let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
        let classify = |m: Result<(), String>| EditError::from_engine_message(m.unwrap_err(), None);
        assert!(matches!(
            classify(model.add_sheet("bad/name")),
            EditError::SheetName(_)
        ));
        assert!(matches!(
            classify(model.add_sheet("Sheet1")),
            EditError::SheetName(_)
        ));
        assert!(matches!(
            classify(model.rename_sheet_by_index(0, "")),
            EditError::SheetName(_)
        ));
        assert!(matches!(
            classify(model.delete_sheet(0)),
            EditError::LastSheet
        ));
        // Without a target cell anything else is the catch-all; with one it is the input's fault.
        assert!(matches!(
            EditError::from_engine_message("boom".into(), None),
            EditError::Engine(m) if m == "boom"
        ));
        let cell = CellRef::new(SheetId::new(0), 0, 0).unwrap();
        assert!(matches!(
            EditError::from_engine_message("boom".into(), Some(cell)),
            EditError::InvalidInput { .. }
        ));
        assert!(matches!(
            EditError::from_engine_message("Invalid sheet".into(), Some(cell)),
            EditError::Ref(RefError::UnknownSheet(_))
        ));
    }

    #[test]
    fn errors_display() {
        assert_eq!(
            RefError::UnknownSheet("S".into()).to_string(),
            "unknown sheet \"S\""
        );
        let e: EditError = RefError::TooLarge.into();
        assert_eq!(e.to_string(), "range too large");
    }
}
