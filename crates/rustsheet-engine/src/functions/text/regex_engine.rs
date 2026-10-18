//! The regular expression engine of the REGEX* functions, with the work of every search counted
//! so one call can be stopped (CLAUDE.md directive 5; T-1272 performance review).
//!
//! A search runs on a lazy DFA that gives up instead of thrashing: once its cache has been
//! cleared [`MIN_CLEARS`] times and it builds a state more often than every [`MIN_BYTES_PER_STATE`]
//! bytes. The DFA's work is measured from its cache (bytes scanned and state memory built), so a
//! simple pattern over a long column costs what it really costs. A search the DFA gives up on
//! (or quits, on a Unicode word boundary) runs on a `PikeVM` instead, charged its span times the
//! NFA's state count before it runs, a bound on its work. Capture groups are resolved by the
//! `PikeVM` over the match alone, as the `regex` crate does. Patterns, matches and replacements
//! follow the `regex` crate exactly: the same syntax, leftmost-first semantics, UTF-8 empty-match
//! rules and `$name` interpolation (all from `regex_automata`, which `regex` is built on).
//!
//! Costs are in units of about a nanosecond of a release build, measured for T-1272:
//! `a[ab]{100}c|b[ab]{100}c` (210 states) on a `PikeVM` costs about 4.2 ns a state-byte; lazy DFA
//! state building about 6 ns a byte built; DFA scanning 1-3 ns a byte.

use super::lifted::to_u64;
use regex_automata::{
    hybrid::{
        dfa::{Cache as DfaCache, Config as DfaConfig, DFA},
        regex::{Cache as RegexCache, Regex as DfaRegex},
    },
    nfa::thompson::{
        self,
        pikevm::{Cache as PikeCache, PikeVM},
        WhichCaptures, NFA,
    },
    util::{captures::Captures, iter::Searcher},
    Anchored, Input, Match, MatchError, MatchKind, PatternID,
};

/// Lazy DFA cache capacity, per direction: the `regex` crate's default.
const DFA_CACHE: usize = 2 << 20;

/// Cache clears before the lazy DFA may give up (the `regex` crate's own setting).
const MIN_CLEARS: usize = 3;

/// The fewest bytes a lazy DFA state must be worth once it may give up (the `regex` crate's).
const MIN_BYTES_PER_STATE: usize = 10;

/// Units charged per byte the lazy DFA scans.
const DFA_BYTE: u64 = 4;

/// Units charged per byte of lazy DFA state memory built.
const BUILT_BYTE: u64 = 8;

/// Units charged per NFA state per byte a `PikeVM` searches.
const PIKE_STATE_BYTE: u64 = 5;

/// Units charged per search, whatever its length: a pattern that matches everywhere does one
/// search a character.
const SEARCH: u64 = 64;

/// Why a pattern did not compile.
pub(super) enum BuildFailure {
    /// Too big for the size limit it was compiled under.
    TooBig,
    /// Not a valid pattern.
    Invalid,
}

/// The work a call has left, in the units above; spent once a charge does not fit.
#[derive(Debug)]
pub(super) struct Spent;

/// A compiled pattern.
pub(super) struct Engine {
    dfa: Option<DfaRegex>,
    pike: PikeVM,
    states: u64,
}

/// The mutable search state of one [`Engine`].
pub(super) struct Caches {
    dfa: Option<RegexCache>,
    pike: PikeCache,
}

impl Engine {
    /// Compiles `pattern` with its NFAs under `size_limit` bytes.
    pub(super) fn new(pattern: &str, size_limit: usize) -> Result<Engine, BuildFailure> {
        let failure = |error: thompson::BuildError| {
            if error.size_limit().is_some() {
                BuildFailure::TooBig
            } else {
                BuildFailure::Invalid
            }
        };
        let forward = NFA::compiler()
            .configure(NFA::config().nfa_size_limit(Some(size_limit)))
            .build(pattern)
            .map_err(failure)?;
        let reverse = NFA::compiler()
            .configure(
                NFA::config()
                    .nfa_size_limit(Some(size_limit))
                    .reverse(true)
                    .which_captures(WhichCaptures::None),
            )
            .build(pattern)
            .map_err(failure)?;
        let states = to_u64(forward.states().len());
        let config = DfaConfig::new()
            .cache_capacity(DFA_CACHE)
            .skip_cache_capacity_check(true)
            .minimum_cache_clear_count(Some(MIN_CLEARS))
            .minimum_bytes_per_state(Some(MIN_BYTES_PER_STATE))
            .unicode_word_boundary(true);
        let fwd = DFA::builder()
            .configure(config.clone())
            .build_from_nfa(forward.clone());
        // The reverse DFA finds where a match starts, configured as `hybrid::regex` does.
        let rev = DFA::builder()
            .configure(
                config
                    .prefilter(None)
                    .specialize_start_states(false)
                    .match_kind(MatchKind::All),
            )
            .build_from_nfa(reverse);
        let dfa = match (fwd, rev) {
            (Ok(fwd), Ok(rev)) => Some(DfaRegex::builder().build_from_dfas(fwd, rev)),
            // Every search then runs on the PikeVM, charged by state count.
            _ => None,
        };
        let pike = PikeVM::new_from_nfa(forward).map_err(|_| BuildFailure::Invalid)?;
        Ok(Engine { dfa, pike, states })
    }

    pub(super) fn create_caches(&self) -> Caches {
        Caches {
            dfa: self.dfa.as_ref().map(DfaRegex::create_cache),
            pike: self.pike.create_cache(),
        }
    }

    /// Captures for one match of this pattern.
    pub(super) fn create_captures(&self) -> Captures {
        self.pike.create_captures()
    }

    /// The index of the capture group named `name`.
    pub(super) fn group_index(&self, name: &str) -> Option<usize> {
        self.pike
            .get_nfa()
            .group_info()
            .to_index(PatternID::ZERO, name)
    }

    /// The number of capture groups of this pattern, the whole match (group 0) included.
    pub(super) fn group_len(&self) -> usize {
        self.pike.get_nfa().group_info().group_len(PatternID::ZERO)
    }

    /// The heap memory the pattern and `caches` hold.
    pub(super) fn memory_usage(&self, caches: &Caches) -> usize {
        self.dfa.as_ref().map_or(0, |dfa| {
            dfa.forward().memory_usage() + dfa.reverse().memory_usage()
        }) + caches.dfa.as_ref().map_or(0, RegexCache::memory_usage)
            + self.pike.get_nfa().memory_usage()
            + caches.pike.memory_usage()
    }
}

/// Bytes the lazy DFA cache `cache` has scanned and state bytes it has built, as far as its
/// counters tell: both reset when the cache is cleared, so a clear is counted as a full cache.
fn dfa_progress(cache: &DfaCache) -> (usize, usize, usize) {
    (
        cache.clear_count(),
        cache.search_total_len(),
        cache.memory_usage(),
    )
}

/// The work between two [`dfa_progress`] readings.
fn dfa_work(before: (usize, usize, usize), after: (usize, usize, usize)) -> u64 {
    let (clears, scanned, built) = if after.0 == before.0 {
        (
            0,
            after.1.saturating_sub(before.1),
            after.2.saturating_sub(before.2),
        )
    } else {
        // What was scanned or built before the last clear is the full caches it cleared.
        (after.0 - before.0, after.1, after.2)
    };
    let scanned = to_u64(scanned);
    let built = to_u64(clears.saturating_mul(DFA_CACHE).saturating_add(built));
    scanned
        .saturating_mul(DFA_BYTE)
        .saturating_add(built.saturating_mul(BUILT_BYTE))
}

/// Searches with one [`Engine`], charging each against the work left.
pub(super) struct Search<'a> {
    pub(super) engine: &'a Engine,
    pub(super) caches: &'a mut Caches,
    pub(super) work: &'a mut u64,
}

impl Search<'_> {
    fn charge(&mut self, units: u64) -> Result<(), Spent> {
        match self.work.checked_sub(units) {
            Some(left) => {
                *self.work = left;
                Ok(())
            }
            None => {
                *self.work = 0;
                Err(Spent)
            }
        }
    }

    /// Charges a `PikeVM` search over `len` bytes.
    fn charge_pike(&mut self, len: usize) -> Result<(), Spent> {
        let len = to_u64(len);
        self.charge(
            len.saturating_mul(self.engine.states)
                .saturating_mul(PIKE_STATE_BYTE),
        )
    }

    /// The leftmost-first match in `input`.
    fn find_in(&mut self, input: &Input<'_>) -> Result<Option<Match>, Spent> {
        self.charge(SEARCH)?;
        if let (Some(dfa), Some(cache)) = (self.engine.dfa.as_ref(), self.caches.dfa.as_mut()) {
            let before = {
                let (fwd, rev) = cache.as_parts();
                (dfa_progress(fwd), dfa_progress(rev))
            };
            let found = dfa.try_search(cache, input);
            let work = {
                let (fwd, rev) = cache.as_parts();
                dfa_work(before.0, dfa_progress(fwd))
                    .saturating_add(dfa_work(before.1, dfa_progress(rev)))
            };
            self.charge(work)?;
            if let Ok(found) = found {
                return Ok(found);
            }
        }
        self.charge_pike(input.end().saturating_sub(input.start()))?;
        Ok(self.engine.pike.find(&mut self.caches.pike, input.clone()))
    }

    /// Fills `caps` from the match `m`, found in `input`.
    fn capture(&mut self, input: &Input<'_>, m: Match, caps: &mut Captures) -> Result<(), Spent> {
        self.charge_pike(m.len())?;
        let input = input
            .clone()
            .span(m.range())
            .anchored(Anchored::Pattern(m.pattern()));
        self.engine.pike.search(&mut self.caches.pike, &input, caps);
        Ok(())
    }

    /// Whether the pattern matches anywhere in `text`.
    pub(super) fn is_match(&mut self, text: &str) -> Result<bool, Spent> {
        Ok(self.find_in(&Input::new(text).earliest(true))?.is_some())
    }

    /// The first match in `text`.
    pub(super) fn find(&mut self, text: &str) -> Result<Option<Match>, Spent> {
        self.find_in(&Input::new(text))
    }

    /// The groups of the first match in `text`, in `caps`; whether there was one.
    pub(super) fn captures(&mut self, text: &str, caps: &mut Captures) -> Result<bool, Spent> {
        let input = Input::new(text);
        match self.find_in(&input)? {
            Some(m) => {
                self.capture(&input, m, caps)?;
                Ok(caps.is_match())
            }
            None => Ok(false),
        }
    }

    /// Calls `each` with every successive non-overlapping match in `text`.
    pub(super) fn for_each_match(
        &mut self,
        text: &str,
        mut each: impl FnMut(Match),
    ) -> Result<(), Spent> {
        let mut searcher = Searcher::new(Input::new(text));
        while let Some(m) = self.advance(&mut searcher)? {
            each(m);
        }
        Ok(())
    }

    /// Calls `each` with the groups of every successive non-overlapping match in `text`.
    pub(super) fn for_each_captures(
        &mut self,
        text: &str,
        caps: &mut Captures,
        mut each: impl FnMut(&Captures),
    ) -> Result<(), Spent> {
        let mut searcher = Searcher::new(Input::new(text));
        while let Some(m) = self.advance(&mut searcher)? {
            self.capture(searcher.input(), m, caps)?;
            each(caps);
        }
        Ok(())
    }

    /// The next match of `searcher`, which keeps the `regex` crate's rule that an empty match
    /// may not end where the previous match did.
    fn advance(&mut self, searcher: &mut Searcher<'_>) -> Result<Option<Match>, Spent> {
        // `find_in` fails only when the work is spent, which the closure's error stands for.
        searcher
            .try_advance(|input| {
                self.find_in(input)
                    .map_err(|Spent| MatchError::gave_up(input.start()))
            })
            .map_err(|_| Spent)
    }
}
