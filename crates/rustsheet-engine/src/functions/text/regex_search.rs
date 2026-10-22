//! One compiled REGEX* pattern, whichever engine runs it (SPEC-0013 section 5): a regular
//! pattern on the automata of [`regex_engine`](super::regex_engine), any other on the
//! backtracker of [`pcre`](super::pcre). Both report byte spans of the text and fill
//! `regex_automata` captures, and both charge the call's one work counter.

use regex_automata::{
    util::captures::{Captures, GroupInfo},
    Match, PatternID,
};

use super::{
    pcre::{BacktrackCache, Backtracker, Program},
    regex_engine::{Caches, Engine, EngineSearch, Spent},
};

/// A compiled pattern and its search caches (the automata's boxed: they are some kilobytes).
pub(super) enum Matcher {
    Automata {
        engine: Box<Engine>,
        caches: Box<Caches>,
    },
    Backtrack {
        program: Program,
        cache: BacktrackCache,
    },
}

impl Matcher {
    /// The heap memory the pattern and its caches hold.
    pub(super) fn memory_usage(&self) -> usize {
        match self {
            Matcher::Automata { engine, caches } => engine.memory_usage(caches),
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
    pub(super) fn is_match(&mut self, text: &str) -> Result<bool, Spent> {
        match self.split() {
            Engines::Automata(mut search) => search.is_match(text),
            Engines::Backtrack(mut bt) => Ok(bt.find_from(text, 0)?.is_some()),
        }
    }

    /// The first match in `text`.
    pub(super) fn find(&mut self, text: &str) -> Result<Option<Match>, Spent> {
        match self.split() {
            Engines::Automata(mut search) => search.find(text),
            Engines::Backtrack(mut bt) => Ok(bt.find_from(text, 0)?.map(to_match)),
        }
    }

    /// The groups of the first match in `text`, in `caps`; whether there was one.
    pub(super) fn captures(&mut self, text: &str, caps: &mut Captures) -> Result<bool, Spent> {
        match self.split() {
            Engines::Automata(mut search) => search.captures(text, caps),
            Engines::Backtrack(mut bt) => {
                if bt.find_from(text, 0)?.is_none() {
                    return Ok(false);
                }
                bt.fill(caps);
                Ok(true)
            }
        }
    }

    /// Fills `caps` with the groups of `m`, a match found in `text`.
    pub(super) fn captures_at(
        &mut self,
        text: &str,
        m: Match,
        caps: &mut Captures,
    ) -> Result<(), Spent> {
        match self.split() {
            Engines::Automata(mut search) => search.captures_at(text, m, caps),
            Engines::Backtrack(mut bt) => {
                // The iteration that found `m` is replayed up to it, so `\G` stands where that
                // search started and the groups are those of `m` itself.
                each_match(&mut bt, text, |bt, found| {
                    if found == m {
                        bt.fill(caps);
                        return false;
                    }
                    found.start() < m.start()
                })
            }
        }
    }

    /// Calls `each` with every successive non-overlapping match in `text`.
    pub(super) fn for_each_match(
        &mut self,
        text: &str,
        mut each: impl FnMut(Match),
    ) -> Result<(), Spent> {
        match self.split() {
            Engines::Automata(mut search) => search.for_each_match(text, each),
            Engines::Backtrack(mut bt) => each_match(&mut bt, text, |_, m| {
                each(m);
                true
            }),
        }
    }

    /// Calls `each` with the groups of every successive non-overlapping match in `text`.
    pub(super) fn for_each_captures(
        &mut self,
        text: &str,
        caps: &mut Captures,
        mut each: impl FnMut(&Captures),
    ) -> Result<(), Spent> {
        match self.split() {
            Engines::Automata(mut search) => search.for_each_captures(text, caps, each),
            Engines::Backtrack(mut bt) => each_match(&mut bt, text, |bt, _| {
                bt.fill(caps);
                each(caps);
                true
            }),
        }
    }

    fn split(&mut self) -> Engines<'_> {
        match self.matcher {
            Matcher::Automata { engine, caches } => Engines::Automata(EngineSearch {
                engine,
                caches,
                work: self.work,
            }),
            Matcher::Backtrack { program, cache } => Engines::Backtrack(Backtracker {
                program,
                cache,
                work: self.work,
                room: self.room,
            }),
        }
    }
}

/// One search on whichever engine the pattern runs on.
enum Engines<'a> {
    Automata(EngineSearch<'a>),
    Backtrack(Backtracker<'a>),
}

fn to_match((start, end): (usize, usize)) -> Match {
    Match::new(PatternID::ZERO, start..end)
}

/// Calls `each` with every successive non-overlapping match of `bt` in `text` while it returns
/// true, by the rule the automata's iterator follows (`regex_automata`'s `Searcher`): the next
/// search starts where a match ended, and an empty match may not end where the previous match
/// did, so the search that found one is repeated one character later. The PCRE2 rule is
/// T-1899's.
fn each_match(
    bt: &mut Backtracker<'_>,
    text: &str,
    mut each: impl FnMut(&Backtracker<'_>, Match) -> bool,
) -> Result<(), Spent> {
    let mut last_end = None;
    let mut retry_from = 0;
    loop {
        let Some((start, end)) = bt.find_from(text, retry_from)? else {
            return Ok(());
        };
        if start == end && Some(end) == last_end {
            // Repeat the search one character on.
            match text.get(retry_from..).and_then(|rest| rest.chars().next()) {
                Some(c) => {
                    retry_from += c.len_utf8();
                    continue;
                }
                None => return Ok(()),
            }
        }
        if !each(bt, to_match((start, end))) {
            return Ok(());
        }
        retry_from = end;
        last_end = Some(end);
    }
}
