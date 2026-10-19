//! How far a leftmost-first `PikeVM` search can read, measured by running the NFA as a set of
//! states (T-1829). A `PikeVM` search is charged before it runs; charging it its whole span made
//! match iteration quadratic in its charge (each search's span is the rest of the text), so `\b`
//! patterns over non-ASCII text, which the lazy DFA cannot search, ran out of work on a few KB.
//! The scan below charges the work it does and returns the end of the bytes the search needs, so
//! the real search runs over that span alone: it then costs about what it really costs, while a
//! truly quadratic iteration (`a.*z|a` over `aaa...`) is still charged every byte it reads.
//!
//! Soundness: the scan keeps every thread the `PikeVM` keeps. Until the `PikeVM` has a match it
//! drops no thread, so the two have the same states; once a thread matches, both stop starting
//! new threads, and the scan keeps the lower-priority threads the `PikeVM` drops. When the scan has
//! a match and no thread left at `at`, neither does the `PikeVM`, whose result is then final, and
//! a search over the span up to `at` gives it (look-around at the span's end still sees the whole
//! haystack). An empty match that splits a UTF-8 character is not a match for the `PikeVM`, which
//! searches on past it, so the scan counts a match only at a character boundary.
//!
//! That retry (`skip_splits_fwd`) runs the `PikeVM` again from one byte later, over the same span.
//! It restarts at or before the scan's first boundary match, and until then the scan starts a
//! thread at every position, so each retry's state sets are subsets of the scan's: the span still
//! holds the retry's match. A pattern that is always anchored (`^abc`) starts threads only at the
//! search's start in the `PikeVM`, while the scan starts one at every position until it has a
//! match, so the scan overcharges it: a visit or two a byte, linear in the span, never less than
//! the work done.

use regex_automata::{
    nfa::thompson::{State, NFA},
    util::primitives::StateID,
    Input,
};

use super::regex_engine::{charge, Spent};

/// The state sets of the scan, kept between searches.
pub(super) struct Reach {
    /// The states consuming a byte at the current position.
    curr: Vec<StateID>,
    /// Those at the next position.
    next: Vec<StateID>,
    /// The closure's work list.
    stack: Vec<StateID>,
    /// For each state, the generation of the set it was last added to.
    marks: Vec<u32>,
    generation: u32,
}

impl Reach {
    pub(super) fn new(nfa: &NFA) -> Self {
        Reach {
            curr: Vec::new(),
            next: Vec::new(),
            stack: Vec::new(),
            marks: vec![0; nfa.states().len()],
            generation: 0,
        }
    }

    /// The heap memory the sets hold.
    pub(super) fn memory_usage(&self) -> usize {
        (self.curr.capacity() + self.next.capacity() + self.stack.capacity())
            * std::mem::size_of::<StateID>()
            + self.marks.capacity() * std::mem::size_of::<u32>()
    }

    /// Starts a new, empty set.
    fn new_set(&mut self) {
        if self.generation == u32::MAX {
            self.marks.fill(0);
            self.generation = 0;
        }
        self.generation += 1;
    }

    /// Adds the states reachable from `start` without consuming a byte at `at` to the `next`
    /// set; whether one of them is a match. `visits` counts every state looked at and every
    /// look-around assertion evaluated.
    fn close(
        &mut self,
        nfa: &NFA,
        haystack: &[u8],
        at: usize,
        start: StateID,
        visits: &mut Visits,
    ) -> bool {
        let mut matched = false;
        self.stack.push(start);
        while let Some(id) = self.stack.pop() {
            visits.states += 1;
            let Some(mark) = self.marks.get_mut(id.as_usize()) else {
                continue;
            };
            if *mark == self.generation {
                continue;
            }
            *mark = self.generation;
            match nfa.state(id) {
                State::ByteRange { .. } | State::Sparse(_) | State::Dense(_) => self.next.push(id),
                State::Look { look, next } => {
                    visits.looks += 1;
                    if nfa.look_matcher().matches(*look, haystack, at) {
                        self.stack.push(*next);
                    }
                }
                State::Union { alternates } => self.stack.extend(alternates.iter().copied()),
                State::BinaryUnion { alt1, alt2 } => self.stack.extend([*alt1, *alt2]),
                State::Capture { next, .. } => self.stack.push(*next),
                State::Fail => {}
                State::Match { .. } => matched = true,
            }
        }
        matched
    }

    /// The end of the span a leftmost-first search of `input` needs, charging `costs` for the
    /// states the scan visits and the look-around assertions it evaluates against `work`.
    /// `input` must be unanchored.
    pub(super) fn scan(
        &mut self,
        nfa: &NFA,
        input: &Input<'_>,
        costs: &Visits,
        work: &mut u64,
    ) -> Result<usize, Spent> {
        let haystack = input.haystack();
        let end = input.end();
        let mut at = input.start();
        let mut matched = false;
        self.next.clear();
        self.new_set();
        loop {
            let mut visits = Visits::default();
            if !matched
                && self.close(nfa, haystack, at, nfa.start_anchored(), &mut visits)
                && is_boundary(haystack, at)
            {
                matched = true;
            }
            std::mem::swap(&mut self.curr, &mut self.next);
            self.next.clear();
            self.new_set();
            let done = (matched && self.curr.is_empty()) || at >= end;
            if !done {
                let byte = haystack.get(at).copied().unwrap_or_default();
                for index in 0..self.curr.len() {
                    let Some(&id) = self.curr.get(index) else {
                        break;
                    };
                    let to = step(nfa, id, byte);
                    if let Some(to) = to {
                        if self.close(nfa, haystack, at + 1, to, &mut visits)
                            && is_boundary(haystack, at + 1)
                        {
                            matched = true;
                        }
                    }
                }
            }
            charge(work, visits.units(costs))?;
            if done {
                return Ok(at.min(end));
            }
            at += 1;
        }
    }
}

/// States visited and look-around assertions evaluated by the scan, or the units each costs.
#[derive(Default)]
pub(super) struct Visits {
    pub(super) states: u64,
    pub(super) looks: u64,
}

impl Visits {
    /// These visits' units at `costs` a visit.
    fn units(&self, costs: &Visits) -> u64 {
        self.states
            .saturating_mul(costs.states)
            .saturating_add(self.looks.saturating_mul(costs.looks))
    }
}

/// The state `id` moves to on `byte`, if it consumes it.
fn step(nfa: &NFA, id: StateID, byte: u8) -> Option<StateID> {
    match nfa.state(id) {
        State::ByteRange { trans } => trans.matches_byte(byte).then_some(trans.next),
        State::Sparse(sparse) => sparse.matches_byte(byte),
        State::Dense(dense) => dense.matches_byte(byte),
        _ => None,
    }
}

/// Whether `at` is a character boundary of the UTF-8 `haystack` (its end included).
fn is_boundary(haystack: &[u8], at: usize) -> bool {
    haystack.get(at).is_none_or(|&b| b & 0xC0 != 0x80)
}
