//! The per-pass cache of lookup search vectors (T-0448). Every exact-match lookup into a range
//! used to evaluate and copy the range's cells, then walk them: 5,000 VLOOKUPs into 20,000 rows
//! cost 10^8 cell reads. Within one pass the cells of a range, once all evaluated, cannot change,
//! so the vector built by the first lookup serves the rest, and its exact-match indexes
//! (`SearchVector::exact_match`, `SearchVector::collated_match`) replace the walks.
//!
//! Why a cached vector reads what a fresh walk would, and when nothing is cached:
//! * It is only stored when building it evaluated every cell of the range to its value for this
//!   pass: no restart or unwind started, no read was blocked (Excel order), and no cell read was
//!   still `Evaluating` (a cycle through the range, whose cell gets its value later)
//!   (`Evaluation::evaluating_reads`).
//! * An evaluated cell keeps its value for the rest of the pass, except where a dynamic array
//!   spills over it or gives it up. The first walk recorded what it saw at every position, so
//!   such a write contradicts that record and restarts the pass, and every pass starts with an
//!   empty cache (`run_pass`). The pass that completes therefore saw no such write after any
//!   cached build.
//! * A read of a circular cell marks its reader circular, so a range holding one is not cached.
//! * Iterative calculation joins each reader of a cycle member to the cycle, which a cached read
//!   would skip: a vector whose build read a member (`Model::cycle_joins` moved) is not stored,
//!   and a cell joins only before it is evaluated, so no cached range holds one (T-0485).
//! * Excel-order passes stop at unevaluated reads and do not cache. Outside a pass (data tables,
//!   conditional formats, iteration sweeps) nothing is cached either.
//!
//! A range of a linked workbook's cache (`LookupTable::External`, T-0474) reads values that never
//! change during a pass, so its vector is cached whenever caching is safe at all.
//!
//! Directive 5: the cached cells across all ranges are capped at [`MAX_CACHED_CELLS`], beyond
//! which lookups walk as before. A vector's two exact-match indexes add at most one key per cell
//! each (for text, its folded units or collation key), so a cached vector with both built takes
//! about three times the memory of its values: still proportional to the capped cells.

use std::collections::HashMap;
use std::sync::Arc;

use crate::evaluation::KeyBuild;
use crate::expressions::types::CellReferenceIndex;
use crate::external_links::{CachedLineEnds, CachedSheet};
use crate::external_reference::ExternalArea;
use crate::model::Model;

use super::lookup_table::SearchVector;

/// Cells all cached vectors of one pass may hold together.
const MAX_CACHED_CELLS: usize = 4_000_000;

/// Ranges shorter than this are cheaper to walk again than to cache.
const MIN_CACHED_CELLS: usize = 64;

/// The evaluated part of a search range.
#[derive(PartialEq, Eq, Hash)]
struct VectorKey {
    /// Its first cell.
    left: (u32, i32, i32),
    /// Its last evaluated cell.
    end: (u32, i32, i32),
    /// The range's full length, the empty tail included.
    len: usize,
    /// Down a column, else along a row.
    down: bool,
    /// The linked workbook's cached sheet the range is on, `None` for this workbook's (`left`
    /// and `end` then hold 0 for the sheet).
    linked: Option<CachedSheet>,
}

/// The search vectors built in the current pass, by range.
#[derive(Default)]
pub(crate) struct SearchCache {
    vectors: HashMap<VectorKey, SearchVector, KeyBuild>,
    cells: usize,
    /// Where each linked sheet's cached cells end, built on the first lookup into it: its size is
    /// bounded by the cache it indexes, so it is kept whatever `cells` says.
    line_ends: HashMap<CachedSheet, Arc<CachedLineEnds>, KeyBuild>,
}

impl SearchCache {
    /// Forgets every vector: a new pass, or cells whose state was reset.
    pub(crate) fn clear(&mut self) {
        self.vectors.clear();
        self.cells = 0;
        self.line_ends.clear();
    }
}

impl Model {
    /// The search vector of `left:end` (`end` the last cell worth evaluating, `len` the range's
    /// full length), from this pass's cache when it is there, else evaluated and, when that was
    /// safe (module docs), cached.
    pub(super) fn cached_search_vector(
        &mut self,
        left: &CellReferenceIndex,
        end: &CellReferenceIndex,
        len: usize,
        down: bool,
    ) -> SearchVector {
        let cells = usize::try_from(if down {
            end.row - left.row + 1
        } else {
            end.column - left.column + 1
        })
        .unwrap_or(0);
        let key = VectorKey {
            left: (left.sheet, left.row, left.column),
            end: (end.sheet, end.row, end.column),
            len,
            down,
            linked: None,
        };
        let cacheable = cells >= MIN_CACHED_CELLS && self.pass_cache_safe();
        if cacheable {
            if let Some(vector) = self.evaluation.search_cache.vectors.get(&key) {
                return vector.clone();
            }
        }
        let evaluating_reads = self.evaluation.evaluating_reads;
        let joins = self.cycle_joins();
        let vector = SearchVector::with_len(self.prepare_array(left, end, down), len);
        let cache = &self.evaluation.search_cache;
        if cacheable
            && cache.cells + cells <= MAX_CACHED_CELLS
            && self.pass_cache_safe()
            && self.evaluation.blocked_on.is_none()
            && self.evaluation.evaluating_reads == evaluating_reads
            && self.cycle_joins() == joins
            && !self.any_circular(left, end)
        {
            let cache = &mut self.evaluation.search_cache;
            cache.cells += cells;
            cache.vectors.insert(key, vector.clone());
        }
        vector
    }

    /// The first column (`down`) or row of a linked workbook's cached `area`: its cells up to the
    /// last one the cache holds, then an empty tail to the area's full length, so every search
    /// probes the positions it would over a range of this workbook (T-0474). From this pass's
    /// cache when it is there (module docs).
    pub(super) fn external_search_vector(
        &mut self,
        area: ExternalArea,
        down: bool,
        cell: CellReferenceIndex,
    ) -> SearchVector {
        let (rows, columns) = area.extent();
        let len = if down { rows } else { columns };
        let key = VectorKey {
            left: (0, area.first_row, area.first_column),
            end: (0, area.first_row, area.first_column),
            len,
            down,
            linked: area.cache,
        };
        let cacheable = self.pass_cache_safe();
        if cacheable {
            if let Some(vector) = self.evaluation.search_cache.vectors.get(&key) {
                return vector.clone();
            }
        }
        let cells = area
            .cache
            .map_or(0, |sheet| self.line_ends(sheet).leading_len(area, down))
            .min(len);
        let values = (0..cells)
            .map(|i| {
                let i = i32::try_from(i).unwrap_or(i32::MAX);
                let (row, column) = if down { (i, 0) } else { (0, i) };
                self.external_area_cell(area, row, column, cell)
            })
            .collect();
        let vector = SearchVector::with_len(values, len);
        let cache = &mut self.evaluation.search_cache;
        if cacheable && cells >= MIN_CACHED_CELLS && cache.cells + cells <= MAX_CACHED_CELLS {
            cache.cells += cells;
            cache.vectors.insert(key, vector.clone());
        }
        vector
    }

    /// [`Model::cached_line_ends`] of `sheet`, built once and then shared by every lookup into it,
    /// so a lookup over a sparse or empty column of a large cache costs no walk of it (CLAUDE.md
    /// directive 5). Kept outside a pass too (data tables, conditional formats): the linked cache
    /// is only written by import, before any evaluation, and each pass starts from an empty cache.
    fn line_ends(&mut self, sheet: CachedSheet) -> Arc<CachedLineEnds> {
        if let Some(ends) = self.evaluation.search_cache.line_ends.get(&sheet) {
            return Arc::clone(ends);
        }
        let ends = Arc::new(self.cached_line_ends(sheet));
        self.evaluation
            .search_cache
            .line_ends
            .insert(sheet, Arc::clone(&ends));
        ends
    }

    /// Whether any cell of `left:end` (one sheet) is known to be circular.
    fn any_circular(&self, left: &CellReferenceIndex, end: &CellReferenceIndex) -> bool {
        let circular = &self.evaluation.circular;
        if circular.is_empty() {
            return false;
        }
        (left.row..=end.row).any(|row| {
            (left.column..=end.column).any(|column| circular.contains(&(left.sheet, row, column)))
        })
    }
}
