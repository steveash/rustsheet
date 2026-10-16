//! The zip package of a workbook, read under resource limits.
//!
//! Workbooks are untrusted (CLAUDE.md directive 5): every part the importer
//! reads goes through [`Package::open`], which charges the bytes actually
//! inflated against one budget for the whole package. The sizes a zip entry
//! declares are attacker-controlled, so they only serve as a fast reject.

use std::cell::Cell;
use std::collections::HashMap;
use std::io::{self, BufReader, Read, Seek};

use super::shared_strings::decode_xlsx_escapes;
use crate::error::{ImportLimit, XlsxError};

/// Resource limits for importing a workbook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportLimits {
    /// Most bytes the parts the importer reads may inflate to, in total.
    pub max_uncompressed_bytes: u64,
    /// Most entries the zip directory may list.
    pub max_zip_entries: u32,
    /// Most characters (Unicode scalar values) in one shared string or cell text, counted after
    /// `_xHHHH_` escapes are decoded. Excel's own cell limit is 32,767 (believed to be UTF-16
    /// code units, so this count is never stricter); what Excel does with a longer one in a file
    /// is not probed yet (T-0004).
    pub max_text_chars: usize,
}

impl Default for ImportLimits {
    fn default() -> Self {
        ImportLimits {
            max_uncompressed_bytes: 256 << 20,
            max_zip_entries: 5_000,
            max_text_chars: 32_767,
        }
    }
}

impl ImportLimits {
    /// Errors if `text` is longer than [`ImportLimits::max_text_chars`].
    pub fn check_text(&self, text: &str) -> Result<(), XlsxError> {
        // Counting characters is only needed once the byte length could exceed the limit.
        if text.len() > self.max_text_chars && text.chars().count() > self.max_text_chars {
            return Err(XlsxError::Limit {
                limit: ImportLimit::TextLength,
                max: u64::try_from(self.max_text_chars).unwrap_or(u64::MAX),
            });
        }
        Ok(())
    }

    /// [`ImportLimits::check_text`] on text as stored in a part, with `_xHHHH_` escapes still
    /// in it: each escape is seven characters that decode to one, so only text longer than the
    /// limit as stored is decoded to be counted.
    pub fn check_escaped_text(&self, stored: &str) -> Result<(), XlsxError> {
        if stored.len() > self.max_text_chars {
            self.check_text(&decode_xlsx_escapes(stored))?;
        }
        Ok(())
    }

    fn bytes_exceeded(&self) -> XlsxError {
        XlsxError::Limit {
            limit: ImportLimit::UncompressedBytes,
            max: self.max_uncompressed_bytes,
        }
    }
}

/// The zip entries of a package by part name, matched the way Excel opens a package: names
/// compare ASCII case-insensitively (OPC part name equivalence, ECMA-376 Part 2) and a
/// backslash in an entry name counts as `/`. Excel for the web opens poi/49609.xlsx, whose
/// entries are named `xl\worksheets\sheet1.xml`, `xl\sharedstrings.xml` and
/// `[content_types].xml` while its relationships point at `/xl/sharedStrings.xml` (oracle
/// probe, T-0112; docs/excel-semantics/xlsx-storage.md).
#[derive(Debug, Default)]
pub struct PartNames {
    /// Normalized part name -> the zip entry names that normalize to it, sorted.
    entries: HashMap<String, Vec<String>>,
}

impl PartNames {
    /// Indexes the entry names of a zip directory (`ZipArchive::file_names`).
    pub fn new<'a>(names: impl IntoIterator<Item = &'a str>) -> Self {
        let mut entries: HashMap<String, Vec<String>> = HashMap::new();
        for name in names {
            entries
                .entry(part_key(name))
                .or_default()
                .push(name.to_string());
        }
        for names in entries.values_mut() {
            names.sort_unstable();
        }
        PartNames { entries }
    }

    /// Indexes the entry names of `archive`'s directory. Fails when an entry name is neither
    /// UTF-8 nor CP437 (zip 9 decodes names on demand).
    pub fn from_archive<R: Read + Seek>(
        archive: &zip::ZipArchive<R>,
    ) -> Result<Self, zip::result::ZipError> {
        let entries = archive.file_names().collect::<Result<Vec<_>, _>>()?;
        Ok(PartNames::new(entries.iter().map(AsRef::as_ref)))
    }

    /// The zip entry that holds the part `name` (such as `xl/workbook.xml`), if any. An entry
    /// named exactly `name` wins; among other equivalent entries (a package OPC calls invalid)
    /// the first in byte order is taken, so the choice is deterministic.
    pub fn entry<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        let names = self.entries.get(&part_key(name))?;
        names
            .iter()
            .find(|entry| *entry == name)
            .or_else(|| names.first())
            .map(String::as_str)
    }
}

/// The part name an entry or a part reference stands for: `/` separators, no leading `/`,
/// ASCII lowercase.
fn part_key(name: &str) -> String {
    name.replace('\\', "/")
        .trim_start_matches('/')
        .to_ascii_lowercase()
}

/// A zip package with a byte budget shared by every part read from it.
pub(crate) struct Package<R> {
    archive: zip::ZipArchive<R>,
    names: PartNames,
    limits: ImportLimits,
    /// Bytes that may still be inflated.
    remaining: Cell<u64>,
    /// Set once a read ran over the budget. Some callers treat any error on an
    /// optional part as "absent"; the importer checks this flag at the end so
    /// such an overrun is still reported.
    exceeded: Cell<bool>,
}

impl<R: Read + Seek> Package<R> {
    pub(crate) fn new(reader: R, limits: ImportLimits) -> Result<Self, XlsxError> {
        let archive = zip::ZipArchive::new(reader)?;
        if u32::try_from(archive.len()).map_or(true, |n| n > limits.max_zip_entries) {
            return Err(XlsxError::Limit {
                limit: ImportLimit::ZipEntries,
                max: u64::from(limits.max_zip_entries),
            });
        }
        let remaining = Cell::new(limits.max_uncompressed_bytes);
        let names = PartNames::from_archive(&archive)?;
        Ok(Package {
            archive,
            names,
            limits,
            remaining,
            exceeded: Cell::new(false),
        })
    }

    pub(crate) fn limits(&self) -> &ImportLimits {
        &self.limits
    }

    /// The typed limit error if any read ran over the byte budget.
    pub(crate) fn overrun(&self) -> Option<XlsxError> {
        self.exceeded.get().then(|| self.limits.bytes_exceeded())
    }

    /// Opens the part `name` for reading.
    pub(crate) fn open(&mut self, name: &str) -> Result<BufReader<Part<'_, R>>, XlsxError> {
        let entry = self
            .names
            .entry(name)
            .ok_or(zip::result::ZipError::FileNotFound)?;
        let file = self.archive.by_name(entry)?;
        if file.size() > self.remaining.get() {
            self.exceeded.set(true);
            return Err(self.limits.bytes_exceeded());
        }
        Ok(BufReader::new(Part {
            file,
            remaining: &self.remaining,
            exceeded: &self.exceeded,
        }))
    }

    /// Opens the part `name`, or `None` if the package has no such part.
    pub(crate) fn open_optional(
        &mut self,
        name: &str,
    ) -> Result<Option<BufReader<Part<'_, R>>>, XlsxError> {
        if self.names.entry(name).is_none() {
            return Ok(None);
        }
        self.open(name).map(Some)
    }
}

/// The part a relationship `target` points to, from a part in the folder `base` (`xl` for the
/// workbook's relationships, `xl/worksheets` for a sheet's). An absolute target starts at the
/// package root; `..` and `.` segments are resolved, never climbing above the root.
pub(crate) fn resolve_target(base: &str, target: &str) -> String {
    let (start, target) = match target.strip_prefix('/') {
        Some(absolute) => ("", absolute),
        None => (base, target),
    };
    let mut segments: Vec<&str> = start.split('/').filter(|s| !s.is_empty()).collect();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            _ => segments.push(segment),
        }
    }
    segments.join("/")
}

/// The folder a part is in, and the relationships part that belongs to it:
/// `xl/worksheets/sheet1.xml` -> (`xl/worksheets`, `xl/worksheets/_rels/sheet1.xml.rels`).
pub(crate) fn part_folder_and_rels(part: &str) -> (&str, String) {
    match part.rsplit_once('/') {
        Some((folder, file)) => (folder, format!("{folder}/_rels/{file}.rels")),
        None => ("", format!("_rels/{part}.rels")),
    }
}

/// One part being inflated, charged against its package's budget.
pub(crate) struct Part<'a, R: Read> {
    file: zip::read::ZipFile<'a, R>,
    remaining: &'a Cell<u64>,
    exceeded: &'a Cell<bool>,
}

impl<R: Read> Read for Part<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.remaining.get();
        // Ask for one byte past the budget so an overrun is distinguishable from an exact fit.
        let cap =
            usize::try_from(remaining.saturating_add(1)).map_or(buf.len(), |c| c.min(buf.len()));
        let read = self.file.read(&mut buf[..cap])?;
        let charged = u64::try_from(read).unwrap_or(u64::MAX);
        if charged > remaining {
            self.exceeded.set(true);
            return Err(io::Error::other("uncompressed size limit exceeded"));
        }
        self.remaining.set(remaining - charged);
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::io::{Cursor, Write};

    fn zip_of(parts: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, content) in parts {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    fn package(
        bytes: Vec<u8>,
        limits: ImportLimits,
    ) -> Result<Package<Cursor<Vec<u8>>>, XlsxError> {
        Package::new(Cursor::new(bytes), limits)
    }

    fn read_all(package: &mut Package<Cursor<Vec<u8>>>, name: &str) -> Result<Vec<u8>, XlsxError> {
        let mut out = Vec::new();
        package.open(name)?.read_to_end(&mut out)?;
        Ok(out)
    }

    #[test]
    fn budget_is_shared_by_all_parts_and_exact_fit_passes() {
        let bytes = zip_of(&[("a", b"12345"), ("b", b"67890")]);
        let limits = ImportLimits {
            max_uncompressed_bytes: 10,
            ..ImportLimits::default()
        };
        let mut pkg = package(bytes.clone(), limits).unwrap();
        assert_eq!(read_all(&mut pkg, "a").unwrap(), b"12345");
        assert_eq!(read_all(&mut pkg, "b").unwrap(), b"67890");
        assert!(pkg.overrun().is_none());

        let mut pkg = package(
            bytes,
            ImportLimits {
                max_uncompressed_bytes: 9,
                ..limits
            },
        )
        .unwrap();
        read_all(&mut pkg, "a").unwrap();
        assert!(read_all(&mut pkg, "b").is_err());
        assert_eq!(pkg.overrun(), Some(pkg.limits().bytes_exceeded()));
    }

    #[test]
    fn too_many_entries_is_a_limit_error() {
        let bytes = zip_of(&[("a", b""), ("b", b""), ("c", b"")]);
        let limits = ImportLimits {
            max_zip_entries: 2,
            ..ImportLimits::default()
        };
        assert_eq!(
            package(bytes, limits).err(),
            Some(XlsxError::Limit {
                limit: ImportLimit::ZipEntries,
                max: 2
            })
        );
    }

    #[test]
    fn missing_optional_part_is_none() {
        let mut pkg = package(zip_of(&[("a", b"x")]), ImportLimits::default()).unwrap();
        assert!(pkg.open_optional("missing").unwrap().is_none());
        assert!(pkg.open_optional("a").unwrap().is_some());
    }

    #[test]
    fn parts_match_entries_case_insensitively_and_across_backslashes() {
        let bytes = zip_of(&[
            ("[content_types].xml", b"t"),
            ("xl\\sharedstrings.xml", b"s"),
            ("xl/workbook.xml", b"exact"),
            ("XL/Workbook.xml", b"other"),
        ]);
        let mut pkg = package(bytes, ImportLimits::default()).unwrap();
        assert_eq!(read_all(&mut pkg, "[Content_Types].xml").unwrap(), b"t");
        assert_eq!(read_all(&mut pkg, "xl/sharedStrings.xml").unwrap(), b"s");
        assert_eq!(read_all(&mut pkg, "/xl/sharedStrings.xml").unwrap(), b"s");
        // An exact name wins over an equivalent one; otherwise byte order decides.
        assert_eq!(read_all(&mut pkg, "xl/workbook.xml").unwrap(), b"exact");
        assert_eq!(read_all(&mut pkg, "XL/Workbook.xml").unwrap(), b"other");
        assert_eq!(read_all(&mut pkg, "xl/WORKBOOK.xml").unwrap(), b"other");
        assert!(pkg.open_optional("xl/styles.xml").unwrap().is_none());
    }

    #[test]
    fn targets_resolve_against_the_part_folder() {
        assert_eq!(
            resolve_target("xl", "worksheets/sheet1.xml"),
            "xl/worksheets/sheet1.xml"
        );
        assert_eq!(
            resolve_target("xl/worksheets", "../comments1.xml"),
            "xl/comments1.xml"
        );
        assert_eq!(
            resolve_target("xl/worksheets", "./c.xml"),
            "xl/worksheets/c.xml"
        );
        assert_eq!(
            resolve_target("xl/worksheets", "/xl/tables/t.xml"),
            "xl/tables/t.xml"
        );
        assert_eq!(resolve_target("xl", "../../../a.xml"), "a.xml");
        assert_eq!(resolve_target("xl", ""), "xl");
        assert_eq!(
            part_folder_and_rels("xl/worksheets/sheet1.xml"),
            (
                "xl/worksheets",
                "xl/worksheets/_rels/sheet1.xml.rels".to_string()
            )
        );
        assert_eq!(
            part_folder_and_rels("sheet.xml"),
            ("", "_rels/sheet.xml.rels".to_string())
        );
    }

    #[test]
    fn text_limit_counts_characters() {
        let limits = ImportLimits {
            max_text_chars: 3,
            ..ImportLimits::default()
        };
        assert!(limits.check_text("abc").is_ok());
        // Three characters, nine bytes
        assert!(limits.check_text("€€€").is_ok());
        // Escapes count as the one character they decode to
        assert!(limits.check_escaped_text("_x000D__x000D__x000D_").is_ok());
        assert!(limits
            .check_escaped_text("_x000D__x000D__x000D__x000D_")
            .is_err());
        assert_eq!(
            limits.check_text("abcd"),
            Err(XlsxError::Limit {
                limit: ImportLimit::TextLength,
                max: 3
            })
        );
    }
}
