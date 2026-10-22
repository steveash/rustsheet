//! The backtracking VM (SPEC-0013 section 3): runs a [`Program`] over a text with one explicit
//! stack of frames and no recursion. Every instruction and every frame is charged to the call's
//! work counter, and the stack may not grow past the heap the call may still hold, so a
//! catastrophic pattern ends in `Spent` (`#ERROR!`) instead of running or allocating unboundedly
//! (CLAUDE.md directive 5).
//!
//! A program compiled for the memoized mode ([`Program::memoized`]) also keeps a visited set of
//! `(instruction, position)` states and never enters one twice, so it runs in time linear in the
//! program times the text (SPEC-0013 section 3, T-2355).
//!
//! The costs are conservative placeholders in `regex_engine`'s units (about a nanosecond each);
//! T-1974 fits them.

use regex_automata::{
    util::{captures::Captures, primitives::NonMaxUsize},
    PatternID,
};

use super::ast::Anchor;
use super::class::literal_class;
use super::grapheme::cluster_end;
use super::program::{Inst, Program};
use crate::functions::text::{
    lifted::to_u64,
    regex_engine::{charge, Spent},
};

/// Units charged per instruction executed.
const STEP: u64 = 8;
/// Units charged per frame pushed, and again per frame an atomic group scans when it closes.
const FRAME: u64 = 8;
/// Units charged per character a backreference compares exactly.
const BACKREF_CHAR: u64 = 8;
/// Units charged per character a case-insensitive backreference compares: folding one
/// character builds and folds a one-character class.
const BACKREF_FOLD_CHAR: u64 = 256;
/// Units charged per start position an unanchored search tries.
const ATTEMPT: u64 = 64;
/// Units charged per word of the visited set cleared: a word is 64 positions of one instruction.
const VISITED_WORD: u64 = 1;
/// Positions (bytes of the text) one chunk of the visited set covers: one bit of a word each.
const CHUNK: usize = 64;

/// One entry of the backtracking stack.
#[derive(Debug, Clone, Copy)]
enum Frame {
    /// Resume at `pc` and `pos`: the other branch of a split.
    Retry { pc: usize, pos: usize },
    /// Restore capture slot `slot` to `old`.
    Slot { slot: usize, old: Option<usize> },
    /// Restore the start of loop `slot`'s iteration to `old`.
    Loop { slot: usize, old: usize },
    /// Where an atomic group opened.
    Barrier,
}

/// The search state of a [`Program`], reused across searches: cleared, not freed.
#[derive(Debug, Default)]
pub(in crate::functions::text) struct BacktrackCache {
    stack: Vec<Frame>,
    slots: Vec<Option<usize>>,
    loops: Vec<usize>,
    visited: Visited,
}

impl BacktrackCache {
    /// The heap memory the cache holds.
    pub(in crate::functions::text) fn memory_usage(&self) -> usize {
        self.stack.capacity() * std::mem::size_of::<Frame>()
            + self.slots.capacity() * std::mem::size_of::<Option<usize>>()
            + self.loops.capacity() * std::mem::size_of::<usize>()
            + self.visited.memory_usage()
    }
}

/// The states the memoized mode has visited: one bit per instruction and byte position, in
/// chunks of [`CHUNK`] positions allocated as the search reaches them. A chunk holds one word
/// per instruction (bit `pos % CHUNK` of word `pc`) and counts only while its stamp is the
/// current one, so a new text or operation clears the set by bumping the stamp, not by
/// touching memory.
#[derive(Debug, Default)]
struct Visited {
    chunks: Vec<Chunk>,
    /// The stamp of the current operation; 0 is never current.
    stamp: u32,
    /// Bytes the chunks' words hold.
    bytes: usize,
}

#[derive(Debug, Default)]
struct Chunk {
    stamp: u32,
    words: Vec<u64>,
}

impl Visited {
    fn memory_usage(&self) -> usize {
        self.chunks.capacity() * std::mem::size_of::<Chunk>() + self.bytes
    }

    /// Starts a new operation: every state is unvisited again.
    fn reset(&mut self) {
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            // Every chunk's stamp could now look current: make each stale for good.
            for chunk in &mut self.chunks {
                chunk.stamp = 0;
            }
            self.stamp = 1;
        }
    }

    /// Marks unvisited again every position from `from` to `to`, both included: where a match
    /// was found, so the next search of the same iteration may cross it again (SPEC-0013
    /// section 3). Marks after `to` stay: they are states that failed before the match's path
    /// was taken, and fail for every later start too.
    fn forget(&mut self, from: usize, to: usize, work: &mut u64) -> Result<(), Spent> {
        for index in from / CHUNK..=to / CHUNK {
            let Some(chunk) = self.chunks.get_mut(index) else {
                break;
            };
            if chunk.stamp != self.stamp {
                continue;
            }
            if (index + 1) * CHUNK <= to + 1 {
                chunk.stamp = 0;
                continue;
            }
            // Keep the bits of the positions after `to` in this chunk: none when `to` is its last
            // position, where the shift by 64 overflows to 0.
            let keep = u64::MAX
                .checked_shl(u32::try_from(to % CHUNK + 1).unwrap_or(u32::MAX))
                .unwrap_or(0);
            charge(work, to_u64(chunk.words.len()).saturating_mul(VISITED_WORD))?;
            for word in &mut chunk.words {
                *word &= keep;
            }
        }
        Ok(())
    }
}

/// One search: a program, its cache, the call's work counter and the stack bytes it may hold.
pub(in crate::functions::text) struct Backtracker<'a> {
    pub(in crate::functions::text) program: &'a Program,
    pub(in crate::functions::text) cache: &'a mut BacktrackCache,
    pub(in crate::functions::text) work: &'a mut u64,
    /// Bytes of stack the search may hold: what the call may still hold.
    pub(in crate::functions::text) room: usize,
}

impl<'a> Backtracker<'a> {
    /// A search of `program` with `cache` for one operation over one text: what the memoized
    /// mode visited in another is forgotten.
    pub(in crate::functions::text) fn new(
        program: &'a Program,
        cache: &'a mut BacktrackCache,
        work: &'a mut u64,
        room: usize,
    ) -> Backtracker<'a> {
        cache.visited.reset();
        Backtracker::resume(program, cache, work, room)
    }

    /// A search of `program` with `cache` that continues the operation `cache` is in: what the
    /// memoized mode visited stays visited (SPEC-0013 section 3).
    pub(in crate::functions::text) fn resume(
        program: &'a Program,
        cache: &'a mut BacktrackCache,
        work: &'a mut u64,
        room: usize,
    ) -> Backtracker<'a> {
        Backtracker {
            program,
            cache,
            work,
            room,
        }
    }

    /// The leftmost match starting at or after `from` (a character boundary of `text`), with `\G`
    /// at `from`: its capture slots stay in the cache. `(start, end)` of the match.
    pub(in crate::functions::text) fn find_from(
        &mut self,
        text: &str,
        from: usize,
    ) -> Result<Option<(usize, usize)>, Spent> {
        let mut start = from;
        loop {
            charge(self.work, ATTEMPT)?;
            if self.attempt(text, start, from, false)? {
                return self.found(from);
            }
            if self.program.anchored {
                return Ok(None);
            }
            match text.get(start..).and_then(|rest| rest.chars().next()) {
                Some(c) => start += c.len_utf8(),
                None => return Ok(None),
            }
        }
    }

    /// One attempt anchored at `at` (a character boundary of `text`) that refuses an empty match,
    /// with `\G` at `at`: PCRE2's `NOTEMPTY_ATSTART | ANCHORED`, the retry after an empty match
    /// (SPEC-0013 section 4). Its capture slots stay in the cache. `(start, end)` of the match.
    ///
    /// In the memoized mode a refused match stays visited: only a match at `at` depends on the
    /// attempt's start, and every later search of the iteration starts after `at`.
    pub(in crate::functions::text) fn retry(
        &mut self,
        text: &str,
        at: usize,
    ) -> Result<Option<(usize, usize)>, Spent> {
        charge(self.work, ATTEMPT)?;
        if self.attempt(text, at, at, true)? {
            return self.found(at);
        }
        Ok(None)
    }

    /// The match the last attempt found, searched for from `from`: in the memoized mode every
    /// position up to its end is unvisited again, so the next search may cross them.
    fn found(&mut self, from: usize) -> Result<Option<(usize, usize)>, Spent> {
        let slot = |i: usize| self.cache.slots.get(i).copied().flatten();
        let found = slot(0).zip(slot(1));
        if let (true, Some((_, end))) = (self.program.memoized, found) {
            self.cache.visited.forget(from, end, self.work)?;
        }
        Ok(found)
    }

    /// Fills `caps` from the slots of the last match [`Backtracker::find_from`] or
    /// [`Backtracker::retry`] found.
    pub(in crate::functions::text) fn fill(&self, caps: &mut Captures) {
        caps.set_pattern(Some(PatternID::ZERO));
        for (out, slot) in caps.slots_mut().iter_mut().zip(&self.cache.slots) {
            *out = slot.and_then(NonMaxUsize::new);
        }
    }

    /// Pushes `frame`, charged; spent once the stack and the visited set would pass
    /// [`Backtracker::room`].
    fn push(&mut self, frame: Frame) -> Result<(), Spent> {
        charge(self.work, FRAME)?;
        let bytes = (self.cache.stack.len() + 1)
            .saturating_mul(std::mem::size_of::<Frame>())
            .saturating_add(self.cache.visited.bytes);
        if bytes > self.room {
            return Err(Spent);
        }
        self.cache.stack.push(frame);
        Ok(())
    }

    /// Marks state `(pc, pos)` visited; whether it already was. A chunk is allocated (or a stale
    /// one cleared) when the search first reaches its positions, charged per word, and spent
    /// once the stack and the visited set would pass [`Backtracker::room`].
    fn visit(&mut self, pc: usize, pos: usize) -> Result<bool, Spent> {
        let words = self.program.insts.len();
        let index = pos / CHUNK;
        let visited = &mut self.cache.visited;
        if visited.chunks.len() <= index {
            visited.chunks.resize_with(index + 1, Chunk::default);
        }
        let stamp = visited.stamp;
        let table = visited.chunks.len() * std::mem::size_of::<Chunk>();
        let Some(chunk) = visited.chunks.get_mut(index) else {
            return Err(Spent);
        };
        if chunk.stamp != stamp {
            charge(self.work, to_u64(words).saturating_mul(VISITED_WORD))?;
            if chunk.words.len() != words {
                let held = visited
                    .bytes
                    .saturating_sub(chunk.words.len() * std::mem::size_of::<u64>())
                    .saturating_add(words.saturating_mul(std::mem::size_of::<u64>()));
                let stack = self.cache.stack.len() * std::mem::size_of::<Frame>() + table;
                if held.saturating_add(stack) > self.room {
                    return Err(Spent);
                }
                visited.bytes = held;
                chunk.words = vec![0; words];
            } else {
                chunk.words.fill(0);
            }
            chunk.stamp = stamp;
        }
        let bit = 1u64 << (pos % CHUNK);
        let Some(word) = chunk.words.get_mut(pc) else {
            return Err(Spent);
        };
        let seen = *word & bit != 0;
        *word |= bit;
        Ok(seen)
    }

    /// One anchored match attempt at `start`, refusing a match that ends there when
    /// `not_empty`; whether it matched.
    fn attempt(
        &mut self,
        text: &str,
        start: usize,
        search_start: usize,
        not_empty: bool,
    ) -> Result<bool, Spent> {
        let program = self.program;
        self.cache.stack.clear();
        self.cache.slots.clear();
        self.cache.slots.resize(program.slots, None);
        self.cache.loops.clear();
        self.cache.loops.resize(program.loops, usize::MAX);
        let (mut pc, mut pos) = (0, start);
        loop {
            charge(self.work, STEP)?;
            // A visited state already failed, or is a loop back to itself with nothing matched
            // on the way, which the Thompson NFA's epsilon closure drops too.
            let seen = program.memoized && self.visit(pc, pos)?;
            let matched = match program.insts.get(pc) {
                _ if seen => false,
                None => false,
                Some(Inst::Match) if not_empty && pos == start => false,
                Some(Inst::Match) => return Ok(true),
                Some(Inst::Char(want)) => match char_at(text, pos) {
                    Some(c) if c == *want => {
                        pos += c.len_utf8();
                        true
                    }
                    _ => false,
                },
                Some(Inst::Class(ranges)) => match char_at(text, pos) {
                    Some(c) if in_ranges(ranges, c) => {
                        pos += c.len_utf8();
                        true
                    }
                    _ => false,
                },
                Some(Inst::Any { dot_all }) => match char_at(text, pos) {
                    Some(c) if *dot_all || c != '\n' => {
                        pos += c.len_utf8();
                        true
                    }
                    _ => false,
                },
                Some(Inst::Grapheme) => match cluster_end(text, pos) {
                    Some((end, chars)) => {
                        // Each character past the first is a step of its own.
                        charge(self.work, STEP.saturating_mul(chars - 1))?;
                        pos = end;
                        true
                    }
                    None => false,
                },
                Some(Inst::Split(first, second)) => {
                    self.push(Frame::Retry { pc: *second, pos })?;
                    pc = *first;
                    continue;
                }
                Some(Inst::Jump(to)) => {
                    pc = *to;
                    continue;
                }
                Some(Inst::Save(slot)) => {
                    self.set_slot(*slot, Some(pos))?;
                    true
                }
                Some(Inst::Close { group, open }) => {
                    let start = self.cache.slots.get(*open).copied().flatten();
                    let first = group.saturating_mul(2);
                    self.set_slot(first, start)?;
                    self.set_slot(first.saturating_add(1), Some(pos))?;
                    true
                }
                Some(Inst::Backref { group, fold }) => {
                    match self.backref(text, pos, *group, *fold)? {
                        Some(end) => {
                            pos = end;
                            true
                        }
                        None => false,
                    }
                }
                Some(Inst::Assert(anchor)) => holds(*anchor, text, pos, search_start),
                Some(Inst::AtomicStart) => {
                    self.push(Frame::Barrier)?;
                    true
                }
                Some(Inst::AtomicEnd) => {
                    self.close_atomic()?;
                    true
                }
                Some(Inst::LoopStart(slot)) => {
                    let old = self.cache.loops.get(*slot).copied().unwrap_or(usize::MAX);
                    self.push(Frame::Loop { slot: *slot, old })?;
                    if let Some(entry) = self.cache.loops.get_mut(*slot) {
                        *entry = pos;
                    }
                    true
                }
                Some(Inst::LoopEnd { slot, back }) => {
                    if self.cache.loops.get(*slot) != Some(&pos) {
                        pc = *back;
                        continue;
                    }
                    true
                }
            };
            if matched {
                pc += 1;
                continue;
            }
            // Backtrack to the latest choice, undoing what was recorded since.
            loop {
                match self.cache.stack.pop() {
                    None => return Ok(false),
                    Some(Frame::Retry { pc: to, pos: at }) => {
                        pc = to;
                        pos = at;
                        break;
                    }
                    Some(Frame::Slot { slot, old }) => {
                        if let Some(entry) = self.cache.slots.get_mut(slot) {
                            *entry = old;
                        }
                    }
                    Some(Frame::Loop { slot, old }) => {
                        if let Some(entry) = self.cache.loops.get_mut(slot) {
                            *entry = old;
                        }
                    }
                    Some(Frame::Barrier) => {}
                }
            }
        }
    }

    /// Sets slot `slot` to `value`, with a frame that restores it on backtracking.
    fn set_slot(&mut self, slot: usize, value: Option<usize>) -> Result<(), Spent> {
        let old = self.cache.slots.get(slot).copied().flatten();
        self.push(Frame::Slot { slot, old })?;
        if let Some(entry) = self.cache.slots.get_mut(slot) {
            *entry = value;
        }
        Ok(())
    }

    /// Closes the innermost atomic group: drops its barrier and the choices above it, keeping
    /// the frames that restore slots and loop starts, so backtracking past the group still
    /// undoes what the group recorded.
    fn close_atomic(&mut self) -> Result<(), Spent> {
        let stack = &mut self.cache.stack;
        let barrier = stack
            .iter()
            .rposition(|frame| matches!(frame, Frame::Barrier))
            .unwrap_or(0);
        let scanned = stack.len().saturating_sub(barrier);
        charge(
            self.work,
            u64::try_from(scanned)
                .unwrap_or(u64::MAX)
                .saturating_mul(FRAME),
        )?;
        let mut kept = barrier;
        for i in barrier..stack.len() {
            if let Some(frame) = stack.get(i).copied() {
                if matches!(frame, Frame::Slot { .. } | Frame::Loop { .. }) {
                    if let Some(entry) = stack.get_mut(kept) {
                        *entry = frame;
                    }
                    kept += 1;
                }
            }
        }
        stack.truncate(kept);
        Ok(())
    }

    /// Matches group `group`'s last text again at `pos`, ignoring case when `fold`; where the
    /// match ends, or `None` (an unset group never matches).
    fn backref(
        &mut self,
        text: &str,
        pos: usize,
        group: usize,
        fold: bool,
    ) -> Result<Option<usize>, Spent> {
        let slot = |i: usize| self.cache.slots.get(i).copied().flatten();
        let first = group.saturating_mul(2);
        let (Some(start), Some(end)) = (slot(first), slot(first.saturating_add(1))) else {
            return Ok(None);
        };
        let Some(captured) = text.get(start..end) else {
            return Ok(None);
        };
        let Some(rest) = text.get(pos..) else {
            return Ok(None);
        };
        let mut rest = rest.chars();
        let mut at = pos;
        for want in captured.chars() {
            charge(
                self.work,
                if fold {
                    BACKREF_FOLD_CHAR
                } else {
                    BACKREF_CHAR
                },
            )?;
            let Some(c) = rest.next() else {
                return Ok(None);
            };
            if c != want && !(fold && same_fold(c, want)) {
                return Ok(None);
            }
            at += c.len_utf8();
        }
        Ok(Some(at))
    }
}

/// The character at byte `pos` of `text`.
fn char_at(text: &str, pos: usize) -> Option<char> {
    text.get(pos..)?.chars().next()
}

/// The character before byte `pos` of `text`.
fn char_before(text: &str, pos: usize) -> Option<char> {
    text.get(..pos)?.chars().next_back()
}

fn in_ranges(ranges: &[regex_syntax::hir::ClassUnicodeRange], c: char) -> bool {
    ranges
        .binary_search_by(|range| {
            if range.end() < c {
                std::cmp::Ordering::Less
            } else if range.start() > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Whether `a` and `b` are equal under simple case folding, the folding the parser gives a
/// case-insensitive literal.
fn same_fold(a: char, b: char) -> bool {
    in_ranges(literal_class(a, true).ranges(), b)
}

fn is_word(c: Option<char>) -> bool {
    c.is_some_and(|c| regex_syntax::try_is_word_character(c).unwrap_or(false))
}

/// Whether `anchor` holds at byte `pos` of `text`, `\G` being `search_start`. The meanings are
/// those the automata translation gives each anchor (`hir.rs`), `$` and `\Z` with PCRE2's final
/// `\n` on every text.
fn holds(anchor: Anchor, text: &str, pos: usize, search_start: usize) -> bool {
    match anchor {
        Anchor::Start => pos == 0,
        Anchor::StartLine => pos == 0 || char_before(text, pos) == Some('\n'),
        Anchor::End => pos == text.len(),
        Anchor::EndLine => pos == text.len() || char_at(text, pos) == Some('\n'),
        Anchor::EndOrFinalLf => {
            pos == text.len() || (pos + 1 == text.len() && char_at(text, pos) == Some('\n'))
        }
        Anchor::WordBoundary => is_word(char_before(text, pos)) != is_word(char_at(text, pos)),
        Anchor::NotWordBoundary => is_word(char_before(text, pos)) == is_word(char_at(text, pos)),
        Anchor::SearchStart => pos == search_start,
    }
}
