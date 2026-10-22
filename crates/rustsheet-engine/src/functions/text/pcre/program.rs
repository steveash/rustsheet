//! A pattern compiled for the backtracker (SPEC-0013 section 3): a flat instruction vector that
//! [`super::backtrack`] runs. Only patterns the automata cannot run come here (`Ast::is_regular`
//! is false).
//!
//! Counted repeats are expanded into copies rather than run by a counter instruction, and the
//! program's size is capped by the caller's compile budget, so `x{1000}` costs 1000 copies of
//! `x` against that budget and `(x{1000}){1000}` is too big. The compiler recurses over the tree,
//! bounded by the parser's nest limit (`parse::NEST_LIMIT`), and stops as soon as the program
//! passes its size limit.

use regex_automata::util::captures::GroupInfo;
use regex_syntax::hir::{ClassUnicode, ClassUnicodeRange};

use super::class::literal_class;

use super::ast::{Anchor, Ast, Reference, RepeatKind};

/// One instruction.
#[derive(Debug, Clone)]
pub(super) enum Inst {
    /// Matches one character exactly.
    Char(char),
    /// Matches one character in the class's sorted ranges.
    Class(Box<[ClassUnicodeRange]>),
    /// `.`: any character, `\n` only when `dot_all`.
    Any { dot_all: bool },
    /// `\X`: one extended grapheme cluster, whole.
    Grapheme,
    /// Tries the first target, then the second on failure.
    Split(usize, usize),
    /// Continues at the target.
    Jump(usize),
    /// Records the position in a slot: group 0's capture slots, or a group's open slot.
    Save(usize),
    /// Closes group `group`, opened at its open slot `open`: only now does the group's capture
    /// (slots `2 * group` and `2 * group + 1`) change, as in PCRE2, so a backreference inside its
    /// own repeated group sees the last completed iteration (`^(a|b\1)+$` is FALSE on `abab`).
    Close { group: usize, open: usize },
    /// Matches again the text a group last matched; fails while the group is unset (PCRE2's
    /// default, without `PCRE2_MATCH_UNSET_BACKREF`).
    Backref { group: usize, fold: bool },
    /// A zero-width assertion.
    Assert(Anchor),
    /// Opens an atomic group: a barrier the matching [`Inst::AtomicEnd`] cuts back to.
    AtomicStart,
    /// Closes an atomic group: drops every choice made inside it.
    AtomicEnd,
    /// Records where an iteration of unbounded loop `k` starts.
    LoopStart(usize),
    /// Ends an iteration of unbounded loop `slot`: back to `back` for another unless the
    /// iteration matched the empty string, which ends the loop (PCRE2's empty-loop rule).
    LoopEnd { slot: usize, back: usize },
    /// A match.
    Match,
}

/// Why a pattern does not compile for the backtracker.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum CompileError {
    /// The program passes its size limit.
    TooBig,
    /// A construct the backtracker does not run yet: lookaround, conditionals and `\K`
    /// (T-2354); `#VALUE!` until then.
    Unsupported,
}

/// A compiled pattern.
#[derive(Debug)]
pub(in crate::functions::text) struct Program {
    pub(super) insts: Vec<Inst>,
    /// Slots: two capture slots a group, the whole match (group 0) included, then one open slot
    /// for each group from 1.
    pub(super) slots: usize,
    /// Unbounded loops, each with its [`Inst::LoopStart`] slot.
    pub(super) loops: usize,
    /// The pattern's groups for `regex_automata`'s `Captures`.
    pub(super) group_info: GroupInfo,
    /// Whether every match starts at the start of the text (`\A`, `^` without `(?m)`).
    pub(super) anchored: bool,
}

impl Program {
    /// The heap memory the program holds.
    pub(in crate::functions::text) fn memory_usage(&self) -> usize {
        self.insts.capacity() * std::mem::size_of::<Inst>()
            + self
                .insts
                .iter()
                .map(|inst| match inst {
                    Inst::Class(ranges) => std::mem::size_of_val(&**ranges),
                    _ => 0,
                })
                .sum::<usize>()
            + self.group_info.memory_usage()
    }

    /// The pattern's groups, the whole match (group 0) included.
    pub(in crate::functions::text) fn group_info(&self) -> &GroupInfo {
        &self.group_info
    }
}

/// Compiles `ast` within `size_limit` bytes of instructions (their class ranges included).
pub(super) fn compile(ast: &Ast, size_limit: usize) -> Result<Program, CompileError> {
    let names = group_names(ast);
    let group_info = GroupInfo::new([std::iter::once(None)
        .chain(names.iter().map(|name| name.as_deref()))
        .collect::<Vec<_>>()])
    .map_err(|_| CompileError::Unsupported)?;
    let mut compiler = Compiler {
        insts: Vec::new(),
        bytes: 0,
        size_limit,
        loops: 0,
        names: &names,
    };
    compiler.push(Inst::Save(0))?;
    compiler.node(ast)?;
    compiler.push(Inst::Save(1))?;
    compiler.push(Inst::Match)?;
    Ok(Program {
        slots: 3 * names.len() + 2,
        loops: compiler.loops,
        insts: compiler.insts,
        group_info,
        anchored: starts_anchored(ast),
    })
}

/// The name of each group, by index from 1.
fn group_names(ast: &Ast) -> Vec<Option<Box<str>>> {
    let mut names = Vec::new();
    let mut stack = vec![ast];
    while let Some(ast) = stack.pop() {
        if let Ast::Capture { index, name, .. } = ast {
            let slot = usize::try_from(*index)
                .unwrap_or(usize::MAX)
                .saturating_sub(1);
            if names.len() <= slot {
                names.resize(slot.saturating_add(1), None);
            }
            if let Some(entry) = names.get_mut(slot) {
                entry.clone_from(name);
            }
        }
        ast.push_children(&mut stack);
    }
    names
}

/// Whether every match of `ast` must start at the start of the text: its first item is `\A`
/// (or `^` without `(?m)`), through groups and concatenations.
fn starts_anchored(ast: &Ast) -> bool {
    let mut ast = ast;
    loop {
        match ast {
            Ast::Anchor(Anchor::Start) => return true,
            Ast::Concat(asts) => match asts.first() {
                Some(first) => ast = first,
                None => return false,
            },
            Ast::Capture { ast: inner, .. } | Ast::Atomic(inner) => ast = inner,
            _ => return false,
        }
    }
}

struct Compiler<'a> {
    insts: Vec<Inst>,
    bytes: usize,
    size_limit: usize,
    loops: usize,
    names: &'a [Option<Box<str>>],
}

impl Compiler<'_> {
    /// Counts `bytes` against the size limit.
    fn charge(&mut self, bytes: usize) -> Result<(), CompileError> {
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > self.size_limit {
            return Err(CompileError::TooBig);
        }
        Ok(())
    }

    /// Appends `inst`; its index.
    fn push(&mut self, inst: Inst) -> Result<usize, CompileError> {
        let extra = match &inst {
            Inst::Class(ranges) => std::mem::size_of_val(&**ranges),
            _ => 0,
        };
        self.charge(std::mem::size_of::<Inst>().saturating_add(extra))?;
        self.insts.push(inst);
        Ok(self.insts.len() - 1)
    }

    /// Points the jump or split at `at` to `target` (its second target when `second`).
    fn patch(&mut self, at: usize, target: usize, second: bool) {
        match self.insts.get_mut(at) {
            Some(Inst::Split(first, _)) if !second => *first = target,
            Some(Inst::Split(_, other)) => *other = target,
            Some(Inst::Jump(to)) => *to = target,
            _ => {}
        }
    }

    fn next(&self) -> usize {
        self.insts.len()
    }

    /// Compiles `ast`; the recursion is bounded by the parser's nest limit. Every visit is
    /// charged an instruction's size, so a repeat of a body that emits nothing
    /// (`(?:(?:){65535}){65535}`) still runs into the size limit instead of looping uncharged.
    fn node(&mut self, ast: &Ast) -> Result<(), CompileError> {
        self.charge(std::mem::size_of::<Inst>())?;
        match ast {
            Ast::Empty => {}
            Ast::Literal { c, fold: false } => {
                self.push(Inst::Char(*c))?;
            }
            Ast::Literal { c, fold: true } => self.class(&literal_class(*c, true))?,
            Ast::Class(class) => self.class(class)?,
            Ast::Dot { dot_all } => {
                self.push(Inst::Any { dot_all: *dot_all })?;
            }
            Ast::Anchor(anchor) => {
                self.push(Inst::Assert(*anchor))?;
            }
            Ast::Capture { index, ast, .. } => {
                // Group g's open slot follows the 2 (n + 1) capture slots.
                let group = usize::try_from(*index).map_err(|_| CompileError::TooBig)?;
                let open = group
                    .checked_add(2 * self.names.len() + 1)
                    .ok_or(CompileError::TooBig)?;
                self.push(Inst::Save(open))?;
                self.node(ast)?;
                self.push(Inst::Close { group, open })?;
            }
            Ast::Concat(asts) => {
                for ast in asts {
                    self.node(ast)?;
                }
            }
            Ast::Alternation(asts) => self.alternation(asts)?,
            Ast::Repeat {
                min,
                max,
                kind,
                ast,
            } => {
                if *kind == RepeatKind::Possessive {
                    self.push(Inst::AtomicStart)?;
                    self.repeat(*min, *max, false, ast)?;
                    self.push(Inst::AtomicEnd)?;
                } else {
                    self.repeat(*min, *max, *kind == RepeatKind::Lazy, ast)?;
                }
            }
            Ast::Backref { reference, fold } => {
                let group = match reference {
                    Reference::Number(number) => {
                        usize::try_from(*number).map_err(|_| CompileError::Unsupported)?
                    }
                    Reference::Name(name) => {
                        self.names
                            .iter()
                            .position(|group| group.as_deref() == Some(&**name))
                            .ok_or(CompileError::Unsupported)?
                            + 1
                    }
                };
                self.push(Inst::Backref { group, fold: *fold })?;
            }
            Ast::Atomic(ast) => {
                self.push(Inst::AtomicStart)?;
                self.node(ast)?;
                self.push(Inst::AtomicEnd)?;
            }
            Ast::Grapheme => {
                self.push(Inst::Grapheme)?;
            }
            Ast::LookAround { .. } | Ast::Conditional { .. } | Ast::KeepOut => {
                return Err(CompileError::Unsupported)
            }
        }
        Ok(())
    }

    fn class(&mut self, class: &ClassUnicode) -> Result<(), CompileError> {
        self.push(Inst::Class(class.ranges().into()))?;
        Ok(())
    }

    /// Alternatives in priority order: each but the last behind a split, all jumping to the end.
    fn alternation(&mut self, asts: &[Ast]) -> Result<(), CompileError> {
        let mut jumps = Vec::new();
        let last = asts.len().saturating_sub(1);
        for (i, ast) in asts.iter().enumerate() {
            if i == last {
                self.node(ast)?;
            } else {
                let split = self.push(Inst::Split(0, 0))?;
                self.patch(split, split + 1, false);
                self.node(ast)?;
                jumps.push(self.push(Inst::Jump(0))?);
                let next = self.next();
                self.patch(split, next, true);
            }
        }
        let end = self.next();
        for jump in jumps {
            self.patch(jump, end, false);
        }
        Ok(())
    }

    /// `ast` repeated `min` to `max` times (no maximum when `None`), greedy unless `lazy`.
    fn repeat(
        &mut self,
        min: u32,
        max: Option<u32>,
        lazy: bool,
        ast: &Ast,
    ) -> Result<(), CompileError> {
        for _ in 0..min {
            self.node(ast)?;
        }
        let Some(max) = max else {
            // L: split(body, exit); body: loop-start; ast; loop-end(back to L); exit:
            let slot = self.loops;
            self.loops += 1;
            let split = self.push(Inst::Split(0, 0))?;
            let body = self.next();
            self.push(Inst::LoopStart(slot))?;
            self.node(ast)?;
            self.push(Inst::LoopEnd { slot, back: split })?;
            let exit = self.next();
            self.set_split(split, body, exit, lazy);
            return Ok(());
        };
        // Each optional copy behind a split to the common exit.
        let mut splits = Vec::new();
        for _ in min..max {
            let split = self.push(Inst::Split(0, 0))?;
            splits.push(split);
            self.node(ast)?;
        }
        let exit = self.next();
        for split in splits {
            self.set_split(split, split + 1, exit, lazy);
        }
        Ok(())
    }

    /// Sets the split at `at` to prefer `body` (greedy) or `exit` (lazy).
    fn set_split(&mut self, at: usize, body: usize, exit: usize, lazy: bool) {
        if let Some(inst) = self.insts.get_mut(at) {
            *inst = if lazy {
                Inst::Split(exit, body)
            } else {
                Inst::Split(body, exit)
            };
        }
    }
}
