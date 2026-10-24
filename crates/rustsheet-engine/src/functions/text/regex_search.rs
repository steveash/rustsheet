//! One compiled REGEX* pattern, whichever engine runs it (SPEC-0013 section 5): a regular
//! pattern on the automata of [`regex_engine`](super::regex_engine), any other on the
//! backtracker of [`pcre`](super::pcre). Both report byte spans of the text and fill
//! `regex_automata` captures, both charge the call's one work counter, and both iterate by
//! PCRE2's global rule ([`each_match`], SPEC-0013 section 4).

use regex_automata::{
    util::captures::{Captures, GroupInfo},
    Match, PatternID,
};

use super::{
    pcre::{Backtrack, BacktrackCache, Backtracker, Program},
    regex_engine::{within_size_limits, BuildFailure, Caches, Engine, EngineSearch, Spent},
};

/// Why a search gave no answer.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum SearchError {
    /// The call's work is spent: `#ERROR!` for the whole call.
    Spent,
    /// The iteration advanced one UTF-16 unit into a surrogate pair, which Excel answers with
    /// `#VALUE!` (probes/regex-pcre2-features D11, D13).
    SplitSurrogate,
}

impl From<Spent> for SearchError {
    fn from(_: Spent) -> Self {
        SearchError::Spent
    }
}

/// A compiled pattern and its search caches (the automata's boxed: they are some kilobytes).
pub(super) enum Matcher {
    /// A regular pattern.
    Automata {
        engine: Box<Engine>,
        caches: Box<Caches>,
        retry: Box<Retry>,
    },
    /// A pattern only the backtracker runs, or a regular one its memoized mode runs.
    Backtrack {
        program: Program,
        cache: BacktrackCache,
    },
}

/// The memoized program a regular pattern's empty-match retry runs on (SPEC-0013 section 4),
/// compiled the first time an iteration needs one: a pattern that cannot match empty never
/// does.
pub(super) enum Retry {
    /// Not needed yet: the pattern to compile.
    Pending(Backtrack),
    /// Compiled, with the visited set its retries share.
    Compiled {
        program: Program,
        cache: BacktrackCache,
    },
}

impl Retry {
    /// The program and its cache, compiled within the call's compile `budget` if this is the
    /// first retry; spent when the budget or every size limit is passed.
    fn compiled(&mut self, budget: &mut usize) -> Result<(&Program, &mut BacktrackCache), Spent> {
        match self {
            Retry::Compiled { program, cache } => Ok((program, cache)),
            Retry::Pending(source) => {
                let program = within_size_limits(budget, |limit| {
                    source.compile(limit).map_err(BuildFailure::from)
                })?
                .ok_or(Spent)?;
                *self = Retry::Compiled {
                    program,
                    cache: BacktrackCache::default(),
                };
                self.compiled(budget)
            }
        }
    }

    /// The heap memory the compiled program and its cache hold.
    fn memory_usage(&self) -> usize {
        match self {
            Retry::Pending(_) => 0,
            Retry::Compiled { program, cache } => program.memory_usage() + cache.memory_usage(),
        }
    }
}

impl Matcher {
    /// A regular pattern compiled for the automata, `retry` its source for the backtracker.
    pub(super) fn automata(engine: Engine, retry: Backtrack) -> Matcher {
        let caches = Box::new(engine.create_caches());
        Matcher::Automata {
            engine: Box::new(engine),
            caches,
            retry: Box::new(Retry::Pending(retry)),
        }
    }

    /// The heap memory the pattern and its caches hold.
    pub(super) fn memory_usage(&self) -> usize {
        match self {
            Matcher::Automata {
                engine,
                caches,
                retry,
            } => engine.memory_usage(caches) + retry.memory_usage(),
            Matcher::Backtrack { program, cache } => program.memory_usage() + cache.memory_usage(),
        }
    }

    fn group_info(&self) -> &GroupInfo {
        match self {
            Matcher::Automata { engine, .. } => engine.group_info(),
            Matcher::Backtrack { program, .. } => program.group_info(),
        }
    }
}

/// Searches with one [`Matcher`], charging each against the work left.
pub(super) struct Search<'a> {
    pub(super) matcher: &'a mut Matcher,
    pub(super) work: &'a mut u64,
    /// Bytes the backtracker's stack may hold: what the call may still hold.
    pub(super) room: usize,
    /// The call's compile budget, which a regular pattern's first retry is compiled within.
    pub(super) budget: &'a mut usize,
}

impl Search<'_> {
    /// The pattern's groups, the whole match (group 0) included.
    pub(super) fn group_info(&self) -> &GroupInfo {
        self.matcher.group_info()
    }

    /// The number of capture groups of this pattern, the whole match (group 0) included.
    pub(super) fn group_len(&self) -> usize {
        self.group_info().group_len(PatternID::ZERO)
    }

    /// Captures for one match of this pattern.
    pub(super) fn create_captures(&self) -> Captures {
        Captures::all(self.group_info().clone())
    }

    /// Whether the pattern matches anywhere in `text`.
    pub(super) fn is_match(&mut self, text: &str) -> Result<bool, SearchError> {
        Ok(match self.split() {
            Engines::Automata(mut automata) => automata.search.is_match(text)?,
            Engines::Backtrack(mut bt) => bt.find_from(text, 0)?.is_some(),
        })
    }

    /// The first match in `text`.
    pub(super) fn find(&mut self, text: &str) -> Result<Option<Match>, SearchError> {
        Ok(match self.split() {
            Engines::Automata(mut automata) => automata.search.find(text)?,
            Engines::Backtrack(mut bt) => bt.find_from(text, 0)?.map(to_match),
        })
    }

    /// The groups of the first match in `text`, in `caps`; whether there was one.
    pub(super) fn captures(
        &mut self,
        text: &str,
        caps: &mut Captures,
    ) -> Result<bool, SearchError> {
        match self.split() {
            Engines::Automata(mut automata) => Ok(automata.search.captures(text, caps)?),
            Engines::Backtrack(mut bt) => {
                if bt.find_from(text, 0)?.is_none() {
                    return Ok(false);
                }
                bt.fill(caps);
                Ok(true)
            }
        }
    }

    /// Fills `caps` with the groups of `m`, a match the iteration over `text` found. The
    /// iteration is replayed up to it: `\G` stands where that search started, and a retry's
    /// match gets the groups the retry found. One pass up to `m` per call: for one match, not
    /// for a loop over them.
    pub(super) fn captures_at(
        &mut self,
        text: &str,
        m: Match,
        caps: &mut Captures,
    ) -> Result<(), SearchError> {
        self.iterate(text, |finder, found| {
            if found.m == m {
                finder.fill(text, found, caps)?;
                return Ok(false);
            }
            // An empty match comes before the retry's match at the same start.
            Ok(found.m.start() <= m.start())
        })
    }

    /// Calls `each` with every successive match in `text`.
    pub(super) fn for_each_match(
        &mut self,
        text: &str,
        mut each: impl FnMut(Match),
    ) -> Result<(), SearchError> {
        self.iterate(text, |_, found| {
            each(found.m);
            Ok(true)
        })
    }

    /// The match at `index` (from 0) of the iteration over `text`, which goes no further: a
    /// surrogate pair after it is no error (probes/regex-iteration-surrogates A4, A6, A8).
    pub(super) fn nth_match(
        &mut self,
        text: &str,
        index: usize,
    ) -> Result<Option<Match>, SearchError> {
        let mut seen = 0usize;
        let mut nth = None;
        self.iterate(text, |_, found| {
            if seen == index {
                nth = Some(found.m);
                return Ok(false);
            }
            seen += 1;
            Ok(true)
        })?;
        Ok(nth)
    }

    /// Calls `each` with the groups of every successive match in `text`.
    pub(super) fn for_each_captures(
        &mut self,
        text: &str,
        caps: &mut Captures,
        mut each: impl FnMut(&Captures),
    ) -> Result<(), SearchError> {
        self.iterate(text, |finder, found| {
            finder.fill(text, found, caps)?;
            each(caps);
            Ok(true)
        })
    }

    fn iterate(
        &mut self,
        text: &str,
        each: impl FnMut(&mut dyn Finder, Found) -> Result<bool, Spent>,
    ) -> Result<(), SearchError> {
        match self.split() {
            Engines::Automata(mut automata) => each_match(&mut automata, text, each),
            Engines::Backtrack(mut bt) => each_match(&mut bt, text, each),
        }
    }

    fn split(&mut self) -> Engines<'_> {
        match self.matcher {
            Matcher::Automata {
                engine,
                caches,
                retry,
            } => Engines::Automata(Automata {
                search: EngineSearch {
                    engine,
                    caches,
                    work: self.work,
                },
                retry,
                budget: self.budget,
                room: self.room,
                retried: false,
            }),
            Matcher::Backtrack { program, cache } => {
                Engines::Backtrack(Backtracker::new(program, cache, self.work, self.room))
            }
        }
    }
}

/// One search on whichever engine the pattern runs on.
enum Engines<'a> {
    Automata(Automata<'a>),
    Backtrack(Backtracker<'a>),
}

/// The automata, with the backtracker for the retries after an empty match.
struct Automata<'a> {
    search: EngineSearch<'a>,
    retry: &'a mut Retry,
    budget: &'a mut usize,
    room: usize,
    /// Whether this operation has retried yet: its first retry forgets what earlier operations
    /// visited.
    retried: bool,
}

/// A match of the iteration, and whether the retry after an empty match found it.
#[derive(Clone, Copy)]
pub(in crate::functions::text) struct Found {
    pub(in crate::functions::text) m: Match,
    retried: bool,
}

/// The searches PCRE2's global iteration is made of, on one engine.
pub(in crate::functions::text) trait Finder {
    /// The leftmost match starting at or after `from`, a character boundary of `text`, with
    /// `\G` at `from`.
    fn find_from(&mut self, text: &str, from: usize) -> Result<Option<Match>, Spent>;

    /// One attempt anchored at `at` that refuses the empty match, with `\G` at `at`: PCRE2's
    /// `NOTEMPTY_ATSTART | ANCHORED`.
    fn retry(&mut self, text: &str, at: usize) -> Result<Option<Match>, Spent>;

    /// Fills `caps` with the groups of `found`, the match the last search returned.
    fn fill(&mut self, text: &str, found: Found, caps: &mut Captures) -> Result<(), Spent>;
}

impl Finder for Automata<'_> {
    fn find_from(&mut self, text: &str, from: usize) -> Result<Option<Match>, Spent> {
        self.search.find_from(text, from)
    }

    fn retry(&mut self, text: &str, at: usize) -> Result<Option<Match>, Spent> {
        let (program, cache) = self.retry.compiled(self.budget)?;
        let work = &mut *self.search.work;
        let mut bt = if self.retried {
            Backtracker::resume(program, cache, work, self.room)
        } else {
            Backtracker::new(program, cache, work, self.room)
        };
        self.retried = true;
        Ok(bt.retry(text, at)?.map(to_match))
    }

    fn fill(&mut self, text: &str, found: Found, caps: &mut Captures) -> Result<(), Spent> {
        if !found.retried {
            return self.search.captures_at(text, found.m, caps);
        }
        // The automata would find the empty match at its start again: the retry's slots hold
        // the groups, untouched while `each_match` calls `each` before its next search.
        let (program, cache) = self.retry.compiled(self.budget)?;
        Backtracker::resume(program, cache, &mut *self.search.work, self.room).fill(caps);
        Ok(())
    }
}

impl Finder for Backtracker<'_> {
    fn find_from(&mut self, text: &str, from: usize) -> Result<Option<Match>, Spent> {
        Ok(Backtracker::find_from(self, text, from)?.map(to_match))
    }

    fn retry(&mut self, text: &str, at: usize) -> Result<Option<Match>, Spent> {
        Ok(Backtracker::retry(self, text, at)?.map(to_match))
    }

    fn fill(&mut self, _: &str, _: Found, caps: &mut Captures) -> Result<(), Spent> {
        Backtracker::fill(self, caps);
        Ok(())
    }
}

fn to_match((start, end): (usize, usize)) -> Match {
    Match::new(PatternID::ZERO, start..end)
}

/// Calls `each` with every successive match of `finder` in `text` while it returns true, by
/// PCRE2's global rule (SPEC-0013 section 4; text.md "REGEX* patterns and match iteration are
/// PCRE2 (T-1688)"): after a non-empty match the next search starts at its end; after an empty
/// match at `p`, one attempt anchored at `p` that refuses the empty match, and if that fails a
/// search one UTF-16 unit on, which inside a surrogate pair is [`SearchError::SplitSurrogate`].
/// Visible to `pcre`'s differential test.
pub(in crate::functions::text) fn each_match(
    finder: &mut dyn Finder,
    text: &str,
    mut each: impl FnMut(&mut dyn Finder, Found) -> Result<bool, Spent>,
) -> Result<(), SearchError> {
    let mut from = 0;
    let mut empty_at = None;
    loop {
        let found = match empty_at.take() {
            Some(at) => match finder.retry(text, at)? {
                Some(m) => Found { m, retried: true },
                None => {
                    let Some(c) = text.get(at..).and_then(|rest| rest.chars().next()) else {
                        return Ok(());
                    };
                    if c.len_utf16() > 1 {
                        return Err(SearchError::SplitSurrogate);
                    }
                    from = at + c.len_utf8();
                    continue;
                }
            },
            None => match finder.find_from(text, from)? {
                Some(m) => Found { m, retried: false },
                None => return Ok(()),
            },
        };
        if !each(finder, found)? {
            return Ok(());
        }
        if found.m.is_empty() {
            empty_at = Some(found.m.start());
        } else {
            from = found.m.end();
        }
    }
}
