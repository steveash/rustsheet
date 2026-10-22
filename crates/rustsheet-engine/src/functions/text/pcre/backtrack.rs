//! The backtracking VM (SPEC-0013 section 3): runs a [`Program`] over a text with one explicit
//! stack of frames and no recursion. Every instruction and every frame is charged to the call's
//! work counter, and the stack may not grow past the heap the call may still hold, so a
//! catastrophic pattern ends in `Spent` (`#ERROR!`) instead of running or allocating unboundedly
//! (CLAUDE.md directive 5).
//!
//! The costs are conservative placeholders in `regex_engine`'s units (about a nanosecond each);
//! T-1974 fits them.

use regex_automata::{
    util::{captures::Captures, primitives::NonMaxUsize},
    PatternID,
};

use super::ast::Anchor;
use super::program::{Inst, Program};
use crate::functions::text::regex_engine::{charge, Spent};

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
}

impl BacktrackCache {
    /// The heap memory the cache holds.
    pub(in crate::functions::text) fn memory_usage(&self) -> usize {
        self.stack.capacity() * std::mem::size_of::<Frame>()
            + self.slots.capacity() * std::mem::size_of::<Option<usize>>()
            + self.loops.capacity() * std::mem::size_of::<usize>()
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

impl Backtracker<'_> {
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
            if self.attempt(text, start, from)? {
                let slot = |i: usize| self.cache.slots.get(i).copied().flatten();
                return Ok(slot(0).zip(slot(1)));
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

    /// Fills `caps` from the slots of the last match [`Backtracker::find_from`] found.
    pub(in crate::functions::text) fn fill(&self, caps: &mut Captures) {
        caps.set_pattern(Some(PatternID::ZERO));
        for (out, slot) in caps.slots_mut().iter_mut().zip(&self.cache.slots) {
            *out = slot.and_then(NonMaxUsize::new);
        }
    }

    /// Pushes `frame`, charged; spent once the stack would pass [`Backtracker::room`].
    fn push(&mut self, frame: Frame) -> Result<(), Spent> {
        charge(self.work, FRAME)?;
        let bytes = (self.cache.stack.len() + 1).saturating_mul(std::mem::size_of::<Frame>());
        if bytes > self.room {
            return Err(Spent);
        }
        self.cache.stack.push(frame);
        Ok(())
    }

    /// One anchored match attempt at `start`; whether it matched.
    fn attempt(&mut self, text: &str, start: usize, search_start: usize) -> Result<bool, Spent> {
        let program = self.program;
        self.cache.stack.clear();
        self.cache.slots.clear();
        self.cache.slots.resize(program.slots, None);
        self.cache.loops.clear();
        self.cache.loops.resize(program.loops, usize::MAX);
        let (mut pc, mut pos) = (0, start);
        loop {
            charge(self.work, STEP)?;
            let matched = match program.insts.get(pc) {
                None => false,
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
                    let old = self.cache.slots.get(*slot).copied().flatten();
                    self.push(Frame::Slot { slot: *slot, old })?;
                    if let Some(entry) = self.cache.slots.get_mut(*slot) {
                        *entry = Some(pos);
                    }
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
        let (Some(start), Some(end)) = (slot(2 * group), slot(2 * group + 1)) else {
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
    let mut class =
        regex_syntax::hir::ClassUnicode::new([regex_syntax::hir::ClassUnicodeRange::new(a, a)]);
    class.case_fold_simple();
    in_ranges(class.ranges(), b)
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
