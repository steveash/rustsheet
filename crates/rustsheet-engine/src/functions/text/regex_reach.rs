//! How far a leftmost-first `PikeVM` search can read, measured by running the NFA as a set of
//! states (T-1829). A `PikeVM` search is charged before it runs; charging it its whole span made
//! match iteration quadratic in its charge (each search's span is the rest of the text), so `\b`
//! patterns over non-ASCII text, which the lazy DFA cannot search, ran out of work on a few KB.
//! The scan below charges the work it does and returns the end of the bytes the search needs, so
//! the real search runs over that span alone: it then costs about what it really costs, while a
//! truly quadratic iteration (`a.*z|a` over `aaa...`) is still charged every byte it reads.
//!
//! Soundness: the scan runs the `PikeVM`'s own search (T-2015). Its sets hold the states the
//! `PikeVM`'s do, match states included, in the same priority order: the closure follows a union's
//! alternates depth first, first alternate first, as `epsilon_closure` does, and the thread
//! started at each position comes after those already running. Stepping a set in that order, a
//! match state ends the step and drops the threads below it, as `nexts` does, and from then on
//! no thread starts; when no thread is left at `at` the `PikeVM` stops there too, with the same
//! match, and a search over the span up to `at` gives it (look-around at the span's end still
//! sees the whole haystack). Cutting below a match also keeps a lazy `.*?` from holding a
//! lower-priority thread to the end of the text, which made iterating it quadratic.
//!
//! An empty match that splits a UTF-8 character is not a match for the `PikeVM`, which retries
//! (`skip_splits_fwd`) from one byte later over the same span. The NFA is built for UTF-8, so a
//! match that ends inside a character is empty and comes from the thread started there. Once the
//! scan meets one, which is before any match at a boundary (after that no thread starts), it
//! stops cutting and keeps every thread, still starting one at each position until a match at a
//! boundary: each retry restarts at or before that match, so its state sets are subsets of the
//! scan's and the span still holds the retry's match. A pattern that is always anchored (`^abc`)
//! starts threads only at the search's start in the `PikeVM`, while the scan starts one at every
//! position until it has a match, so the scan overcharges it: a visit or two a byte, linear in
//! the span, never less than the work done.

use regex_automata::{
    nfa::thompson::{State, NFA},
    util::primitives::StateID,
    Input,
};

use super::regex_engine::{charge, Spent};

/// The state sets of the scan, kept between searches.
pub(super) struct Reach {
    /// The states consuming a byte, or matching, at the current position, in priority order.
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
    /// set, in priority order. `visits` counts every state looked at and every look-around
    /// assertion evaluated.
    fn close(
        &mut self,
        nfa: &NFA,
        haystack: &[u8],
        at: usize,
        start: StateID,
        visits: &mut Visits,
    ) {
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
                State::ByteRange { .. }
                | State::Sparse(_)
                | State::Dense(_)
                | State::Match { .. } => self.next.push(id),
                State::Look { look, next } => {
                    visits.looks += 1;
                    if nfa.look_matcher().matches(*look, haystack, at) {
                        self.stack.push(*next);
                    }
                }
                // Pushed last first, so the first alternate is followed first.
                State::Union { alternates } => self.stack.extend(alternates.iter().rev().copied()),
                State::BinaryUnion { alt1, alt2 } => self.stack.extend([*alt2, *alt1]),
                State::Capture { next, .. } => self.stack.push(*next),
                State::Fail => {}
            }
        }
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
        // After an empty match at the end the next search starts past it: the `PikeVM` reads
        // nothing, and look-around past the haystack would panic.
        if input.is_done() {
            return Ok(end);
        }
        let mut at = input.start();
        // A match at a character boundary: no thread starts after it.
        let mut matched = false;
        // An empty match inside a character was met: no thread is cut from then on.
        let mut split = false;
        self.next.clear();
        self.new_set();
        loop {
            let mut visits = Visits::default();
            if !matched {
                self.close(nfa, haystack, at, nfa.start_anchored(), &mut visits);
            }
            std::mem::swap(&mut self.curr, &mut self.next);
            self.next.clear();
            self.new_set();
            let done = (matched && self.curr.is_empty()) || at >= end;
            if done {
                charge(work, visits.units(costs))?;
                return Ok(at.min(end));
            }
            let byte = haystack.get(at).copied().unwrap_or_default();
            for index in 0..self.curr.len() {
                let Some(&id) = self.curr.get(index) else {
                    break;
                };
                if let State::Match { .. } = nfa.state(id) {
                    if !is_boundary(haystack, at) {
                        split = true;
                    } else {
                        matched = true;
                        // The `PikeVM` drops the threads below a match.
                        if !split {
                            break;
                        }
                    }
                } else if let Some(to) = step(nfa, id, byte, &mut visits) {
                    self.close(nfa, haystack, at + 1, to, &mut visits);
                }
            }
            charge(work, visits.units(costs))?;
            at += 1;
        }
    }
}

/// States visited, look-around assertions evaluated and sparse transitions walked by the scan,
/// or the units each costs.
#[derive(Default)]
pub(super) struct Visits {
    pub(super) states: u64,
    pub(super) looks: u64,
    pub(super) transitions: u64,
}

impl Visits {
    /// These visits' units at `costs` a visit.
    fn units(&self, costs: &Visits) -> u64 {
        self.states
            .saturating_mul(costs.states)
            .saturating_add(self.looks.saturating_mul(costs.looks))
            .saturating_add(self.transitions.saturating_mul(costs.transitions))
    }
}

/// The state `id` moves to on `byte`, if it consumes it. A `Sparse` state is walked as
/// `SparseTransitions::matches_byte` walks it (in order, up to the first range past `byte`),
/// counting each range looked at: a large Unicode class costs a visit many times over (T-1988).
fn step(nfa: &NFA, id: StateID, byte: u8, visits: &mut Visits) -> Option<StateID> {
    match nfa.state(id) {
        State::ByteRange { trans } => trans.matches_byte(byte).then_some(trans.next),
        State::Sparse(sparse) => {
            for trans in &sparse.transitions {
                visits.transitions += 1;
                if trans.start > byte {
                    break;
                }
                if trans.matches_byte(byte) {
                    return Some(trans.next);
                }
            }
            None
        }
        State::Dense(dense) => dense.matches_byte(byte),
        _ => None,
    }
}

/// Whether `at` is a character boundary of the UTF-8 `haystack` (its end included).
fn is_boundary(haystack: &[u8], at: usize) -> bool {
    haystack.get(at).is_none_or(|&b| b & 0xC0 != 0x80)
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use regex_automata::{
        nfa::thompson::{pikevm::PikeVM, NFA},
        util::iter::Searcher,
        Input,
    };

    use super::{Reach, Visits};

    /// The units `scan` charges `pattern` over `text` at one unit a sparse transition.
    fn transitions(pattern: &str, text: &str) -> u64 {
        let nfa = NFA::new(pattern).unwrap();
        let mut work = u64::MAX;
        let costs = Visits {
            transitions: 1,
            ..Visits::default()
        };
        Reach::new(&nfa)
            .scan(&nfa, &Input::new(text), &costs, &mut work)
            .unwrap();
        u64::MAX - work
    }

    #[test]
    fn scan_charges_the_sparse_transitions_it_walks() {
        assert_eq!(transitions("x", "xxxx"), 0);
        // A large Unicode class is a `Sparse` state walked range by range on every visit.
        assert!(transitions(r"[\w\s\d]", "日") > 1);
        assert!(transitions(r"[\w\s\d]{3}q", "日日日q") > transitions(r"[\w\s\d]", "日"));
    }

    /// Every match `PikeVM` iteration finds in `text`, as `Search::for_each_match` iterates
    /// (a search after an empty match starts a byte later, maybe inside a character), each
    /// search run over the span the scan measured (`spans`) or over the rest of the text.
    fn matches(pattern: &str, text: &str, spans: bool) -> Vec<(usize, usize)> {
        let pike = PikeVM::new(pattern).unwrap();
        let mut cache = pike.create_cache();
        let mut reach = Reach::new(pike.get_nfa());
        let mut searcher = Searcher::new(Input::new(text));
        let mut found = Vec::new();
        while let Some(m) = searcher.advance(|input| {
            let mut work = u64::MAX;
            let end = if spans {
                reach
                    .scan(pike.get_nfa(), input, &Visits::default(), &mut work)
                    .unwrap()
            } else {
                input.end()
            };
            Ok(pike.find(&mut cache, input.clone().span(input.start()..end)))
        }) {
            found.push((m.start(), m.end()));
        }
        found
    }

    // T-2015: the scan cuts the threads below a match, as the `PikeVM` does; over random
    // patterns and texts, searching the measured span finds what searching the rest finds.
    #[test]
    fn scan_span_keeps_every_match() {
        const ATOMS: [&str; 16] = [
            "a", "b", "é", " ", ".", r"\b", r"\B", "(?:)", "[ab]", r"\w", "^", "$", "a|b", "é|",
            "(a|é)", "(?m:^)",
        ];
        const SUFFIXES: [&str; 7] = ["", "*", "*?", "+", "+?", "?", "??"];
        const CHARS: [char; 5] = ['a', 'b', 'é', ' ', '日'];
        let mut rng = StdRng::seed_from_u64(2015);
        for _ in 0..3000 {
            let mut pattern = String::new();
            for _ in 0..rng.gen_range(1..5) {
                if rng.gen_bool(0.2) {
                    pattern.push('|');
                }
                let atom = ATOMS[rng.gen_range(0..ATOMS.len())];
                pattern.push_str(&format!("(?:{atom})"));
                pattern.push_str(SUFFIXES[rng.gen_range(0..SUFFIXES.len())]);
            }
            let text: String = (0..rng.gen_range(0..12))
                .map(|_| CHARS[rng.gen_range(0..CHARS.len())])
                .collect();
            assert_eq!(
                matches(&pattern, &text, true),
                matches(&pattern, &text, false),
                "{pattern:?} over {text:?}"
            );
        }
    }

    // T-2015: a lazy pattern's lower-priority greedy thread is cut once the lazy one matches,
    // so the scan stops at the match instead of reading to the end of the text.
    #[test]
    fn scan_stops_at_a_lazy_match() {
        let nfa = NFA::new(r"\bab.*?\b").unwrap();
        let text = "é ab ".repeat(100);
        let mut work = u64::MAX;
        let end = Reach::new(&nfa)
            .scan(&nfa, &Input::new(&text), &Visits::default(), &mut work)
            .unwrap();
        assert!(end < 10, "{end}");
    }
}
