# SPEC-0013 — A PCRE2-compatible matcher for the REGEX functions

- **Status:** accepted
- **Milestone / lane:** M7 / functions-text
- **Tasks:** T-1898 (this spec), T-1972, T-1973, T-1900, T-1899, T-1974

## Problem
REGEXTEST, REGEXEXTRACT and REGEXREPLACE use PCRE2 in Excel (docs/excel-semantics/text.md "REGEX* patterns and match
iteration are PCRE2 (T-1688)"). Ours use the `regex` crate's dialect on `regex-automata`
(crates/rustsheet-engine/src/functions/text/regex_engine.rs). That costs us cells in three ways:

1. **Pattern features the `regex` crate lacks.** These are rejected as `#VALUE!` where Excel matches:
   backreferences `\1` `\g1` `\k<n>`, lookahead and lookbehind, atomic groups `(?>..)`, `\K`, conditionals
   `(?(1)b|c)`, comments `(?#..)`, `\Q..\E`, and scoped flags `(?-i)` (probes/regex-pcre2-features C1-C19). Other
   features are read with a different meaning: `a++` is read as `a+` followed by `+`, so `a++a` matches `aaa`, which
   Excel's `#N/A` says it does not (C11, B27-B29). POSIX classes are ASCII-only where Excel's are Unicode
   (`REGEXTEST("é","[[:alpha:]]")`, regexreplace-limit-and-syntax A23).
2. **Global iteration.** After an empty match, PCRE2 first tries a non-empty match anchored at the same position, and
   only then advances one character. Rust's leftmost-first iterator has no "not empty at start" search, so
   `REGEXREPLACE("aaa","a*?","b")` gives `bababab` where Excel gives `bbbbbbb` (t1626-regexreplace C34, D1-D12).
3. **Positions.** Excel iterates UTF-16 units: an empty pattern over text with a surrogate pair is `#VALUE!` (D11,
   D13). We iterate UTF-8 code points.

Each of these is a known failure in eval/baseline/probes.txt today.

## Goals / non-goals
Goals:
- One PCRE2 dialect for every REGEX* call: one parser, one meaning for every construct, on every target (wasm32
  included).
- PCRE2's leftmost, priority-ordered (backtracking) match semantics, including the empty-match iteration rule.
- Directive 5 for every pattern:
  - no recursion on pattern or text size;
  - a work budget charged to the call budget the current engine already uses (2^30 units, text.md "Lifted TEXTSPLIT,
    TEXTBEFORE, TEXTAFTER and REGEX* calls are capped");
  - bounded memory.
- No regression in speed for the patterns the `regex` crate already serves. `SUM(--REGEXTEST(REPT("y",200)&SEQUENCE(100000),"x"))` and the
  T-1272 benchmarks keep their cost.

Non-goals:
- PCRE2's JIT, callouts, `(*VERB)` control verbs, or any construct Excel has not been shown to accept. Each of these
  is `#VALUE!` until a probe shows otherwise.
- The REGEXREPLACE replacement syntax. It is already Excel's (T-1688, functions/text/regex_replacement.rs).
- The result-length caps (T-1817 and T-1829 own them).

## Excel behavior to match
Evidence already in eval (VERIFIED, excel-graph):
- The features in text.md "REGEX* patterns and match iteration are PCRE2 (T-1688)":
  - backreferences, also case-insensitive (`(a)\1` matches `Aa` with case_sensitivity 1);
  - a reference to a missing group is `#VALUE!` (`(a)\2`);
  - lookahead and lookbehind, where a variable-length lookbehind `(?<=a+)` is `#VALUE!`;
  - possessive and atomic groups that cannot match give `#N/A` from REGEXEXTRACT;
  - `\K`, conditionals, comments, `\Q..\E`, `(?x)`, and scoped `(?-i)`.
- Global iteration as above: REGEXEXTRACT's all-matches mode and REGEXREPLACE's occurrence count the same matches
  (D1-D12).
- Case-insensitive matching uses Unicode simple case folding: `ſ` matches `s` and the Kelvin sign matches `k`
  (probes/regex-optional-args A24-A26). `\w`, `\b`, `\pL` and `[[:alpha:]]` are Unicode (text.md line "Unicode: ...").
- An invalid pattern is `#VALUE!` in its own lifted element only (text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX*
  lift ...").

Unknowns, probed first by T-1972 (case probes/regex-pcre2-limits):
- **Excel's match limit.** What do `REGEXTEST(REPT("a",n),"(a|a)*b")` for n = 20, 25, 30, 1000 and
  `REGEXTEST(REPT("a",30),"(a*)*b")` give: FALSE, an error, or a slow answer? PCRE2 stops a match at its match limit
  (10,000,000 by default) and its heap limit (20 MB). PCRE2's auto-possessification may also avoid the blow-up
  altogether.
- **Nesting.** Does `REPT("(",250)&"a"&REPT(")",250)` compile, and is 251 levels `#VALUE!`? PCRE2's default
  parentheses nest limit is 250.
- **Bounded variable-length lookbehind.** Is `(?<=a{1,3})b` accepted? PCRE2 10.43 accepts it and older versions do
  not.
- **Recursion and subroutines.** `(?R)`, `(?1)`, `(?&name)`, for example `REGEXTEST("((()))","^(\((?1)*\))$")`.
- **Escapes and classes:**
  - `.` on a supplementary character (one match or two);
  - `\X`, `\R`, `\h`, `\v`, `\N`, `\x{1F600}`;
  - `\p{Greek}` and `\p{Han}` (scripts);
  - `[[:^alpha:]]`;
  - `\d` on Arabic-Indic digits (is `\d` Unicode, as UCP mode makes it?).
- **Newlines and anchors.** Do `$` and `\Z` match before a final `CHAR(10)`? Does `.` match `CHAR(10)` and
  `CHAR(13)`? What do `(?s)`, `(?m)` and `(?n)` do?
- **Verbs and callouts.** `(*SKIP)(*FAIL)`, `(*UTF)`, `(?C1)`: `#VALUE!` or accepted.

The spec decides the design; the probe decides constants and accept/reject lists. Each probed answer goes into
text.md in the probe task. Where we cannot match Excel's answer under directive 5 (a pattern Excel answers after
millions of steps), the design below gives a `DEVIATION` entry (decision 0014).

## Design

### Overview
```
pattern ──► pcre::parse ──► Ast ──┬─ regular? ──► pcre::to_hir ──► regex_engine::Engine   (lazy DFA / PikeVM, as today)
                                   └─ otherwise ─► pcre::compile ──► pcre::Program ──► pcre::Backtracker
```
New module `crates/rustsheet-engine/src/functions/text/pcre/`:
- `parse.rs`: the PCRE2 pattern grammar;
- `ast.rs`;
- `hir.rs`: AST to `regex_syntax::hir::Hir`;
- `program.rs`: AST to backtracking program;
- `backtrack.rs`: the VM;
- `mod.rs`: `Pattern`, the public face to regex_functions.rs.

The current `Engine` stays as is. It is fed a `Hir` built by our translator instead of a pattern string parsed by
`regex-syntax`, so the `regex` crate's syntax is no longer RustSheet's dialect. Only its automata are used.

### 1. One parser: `pcre::parse`
- Hand-written and iterative. It keeps an explicit group stack, no recursion, and its nesting depth is capped at
  the probed nest limit, assumed 250 until the probe answers.
- Its input is the pattern as Rust `&str`. A pattern with a lone surrogate follows SPEC-0010; no special case is
  needed here, because a lone surrogate is a literal.
- It produces `Ast` nodes:

  | node kind | variants |
  |---|---|
  | literals and escapes | literal, class, `.` |
  | anchors | `^`, `$`, `\A`, `\z`, `\Z`, `\b`, `\B`, `\G` |
  | groups | capture (number and name), non-capture |
  | operators | concatenation, alternation |
  | repetition | greedy, lazy, possessive |
  | references | backreference by number, relative or name; case-insensitive per its scope |
  | assertions | lookahead and lookbehind, both positive and negative |
  | other | atomic group, conditional (on a group or an assertion), `\K` |

- It handles these while parsing, so they never reach the AST: `\Q..\E`, comments, `(?x)`, and inline flags with
  their scope (`i`, `m`, `s`, `x`, `n`, `-`).
- Every construct the parser does not know is `#VALUE!` (`BuildFailure::Invalid`), so an unprobed feature fails
  closed.
- The parser decides the PCRE2 compile-time errors Excel shows as `#VALUE!`:
  - a reference to a missing group;
  - a lookbehind that is not fixed-length per alternative, or bounded once probed;
  - unbalanced parentheses;
  - a bad quantifier.
- Character classes build `regex_syntax::hir::ClassUnicode` (or `ClassBytes`) through the same Unicode tables the
  current engine uses. That includes `case_fold_simple` for case-insensitive matching, so both engines share one
  class and one case-folding definition. POSIX classes are mapped to their Unicode equivalents, which fixes
  `[[:alpha:]]` (A23).

### 2. Regular patterns stay on the automata
`Ast::is_regular` holds when no node is one of these:
- a backreference;
- a lookaround;
- an atomic group;
- a possessive quantifier;
- a conditional;
- `\K`;
- `\G`;
- an anchor whose meaning the automata cannot express.

For a regular pattern, Perl-style priority matching and `regex-automata`'s leftmost-first semantics pick the same
match and the same captures. That is the property the `regex` crate is built on, and the T-1900 differential test
pins it (see "Test plan"). So a regular pattern is translated to `Hir` and runs on today's `Engine`, with its
existing cost model and caches.

### 3. The backtracker: `pcre::Program` and `pcre::Backtracker`
- **Program.** The AST compiles to a flat instruction vector:

  | instruction | role |
  |---|---|
  | `Char` / `Class` | match one character |
  | `Split(primary, alt)` | priority choice |
  | `Jump` | jump |
  | `Save(slot)` | record a capture slot |
  | `Backref(group, fold)` | match a group's text again |
  | `Assert(kind)` | anchors and word boundaries |
  | `LookStart(kind, len)` / `LookEnd` | lookaround |
  | `AtomicStart` / `AtomicEnd` | atomic groups; possessive `x*+` compiles as `(?>x*)` |
  | `CondGroup(group, yes, no)` / `CondAssert` | conditionals |
  | `KeepOut` | `\K` |
  | `RepeatCounter` | counted repetition, so `x{1000}` does not need 1000 copies |
  | `Match` | a match is found |

  The program size is bounded by the pattern length times a constant. It is charged against the existing compile
  budget (128 MiB) like an NFA.
- **VM.**
  - One explicit stack of frames, each one of:
    - `Retry(pc, pos)`;
    - `RestoreSlot(slot, old)`;
    - `RestoreCounter(id, old)`;
    - `AtomicBarrier(depth)`;
    - `LookBarrier(...)`.
  - No Rust recursion anywhere.
  - Lookaround runs as a sub-match on the same stack, delimited by a barrier frame. A lookbehind of fixed length L
    starts at `pos - L` (in characters).
  - An atomic or possessive group, once matched, pops the stack down to its barrier.
  - Empty-loop protection follows PCRE2's rule: an iteration of an unbounded repeat that matched the empty string
    stops the loop. It is enforced with a per-loop "position at iteration start" slot, as in PCRE2.
- **Budget (directive 5).**
  - Each executed instruction costs `STEP` units and each pushed frame `FRAME` units. Both constants are fitted in
    the bench task against the T-1272 unit (about a nanosecond).
  - Both are charged to the call's existing `Spent` budget through the same `Engine`-level accounting, so one call
    has one budget whichever engine runs.
  - The stack is capped in bytes. It counts toward the 64 MiB the call may hold.
  - Running out of either is `#ERROR!`, as the current budget is. If the probe shows Excel answering a pattern that
    exhausts our budget, that pattern is a `DEVIATION` (text.md entry plus decision 0014). If Excel itself fails at
    its match limit, we return the probed error at a fitted equivalent step count when that count is within
    budget.
  - Memoization (bit-set of `(pc, pos)` visited) is not used. PCRE2 does not have it, and with backreferences it
    changes which match is found.

### 4. Positions and iteration (`pcre::Pattern::find_iter`)
- Both engines search Rust `&str` byte offsets, and each match is converted to UTF-16 offsets only where Excel's
  semantics read positions.
- One iterator implements PCRE2's global rule for every pattern:
  1. after a non-empty match ending at `e`, search from `e`;
  2. after an empty match at `p`, first search anchored at `p` with "not empty at start" (PCRE2's
     `NOTEMPTY_ATSTART | ANCHORED`);
  3. if that fails, advance one UTF-16 unit and search again.
- The automata cannot express step 2, so step 2 always runs on the backtracker, also for a regular pattern. It is
  one anchored match attempt per empty match, compiled lazily the first time it is needed and cached with the
  pattern.
- Advancing one UTF-16 unit from a position before a supplementary character lands inside its surrogate pair. Excel
  then returns `#VALUE!` (D11, D13), and so do we: the iterator returns a `SplitSurrogate` error, mapped to
  `#VALUE!` by regex_functions.rs.
- `REGEXTEST` does no iteration, `REGEXEXTRACT` modes 0 and 2 take the first match, and mode 1 and `REGEXREPLACE`
  iterate. Occurrence counting is over the same iterator (D7-D10).

### 5. API and errors
`regex_functions.rs` changes in one place: it builds `pcre::Pattern::new(pattern, case_insensitive, limits)` instead
of `Engine::new`. `Pattern` exposes the three operations the functions need:
- `is_match(&mut Caches, text, &mut Budget)`;
- `captures_at(..)`;
- `find_iter(..)`.

`Pattern` owns either an `Engine` plus a lazily built `Program` (for regular patterns), or a `Program` alone.
`BuildFailure` keeps its two cases (`TooBig`, `Invalid`). The search side gains `SplitSurrogate` beside `Spent`.
No public (crate `rustsheet`) API changes.

### 6. Dependencies
No new crate enters the tree. The parser, compiler and VM are ours. The class and case-folding tables and the `Hir`
type come from `regex-syntax`, which is already in Cargo.lock through `regex` and `regex-automata` (its `syntax`
feature). Step 2 makes `regex-syntax` a direct dependency of rustsheet-engine, pinned to the version already locked,
and feeds `Hir` to `regex_automata::nfa::thompson::Compiler::build_from_hir`. If `cargo deny` or the dependency
policy treats a new direct edge as a new dependency, step 2 adds the decision record.

## Alternatives considered
- **The `pcre2` crate (C PCRE2 by FFI).** Exact semantics for free, but rejected:
  - it adds a C build and `unsafe` FFI to the engine;
  - it does not build on wasm32-unknown-unknown without a C toolchain for that target;
  - it gives up our work accounting, because PCRE2's match limit counts its own units and cannot share the call
    budget;
  - it puts untrusted patterns into C code (directive 5, supply chain).
- **`fancy-regex`.** A pure-Rust backtracker that delegates regular sub-patterns to `regex`. Rejected:
  - its dialect is the `regex` crate's plus extensions, not PCRE2's, so the dialect problem stays (meaning of `\Q`,
    POSIX classes, flags);
  - its iteration is Rust's;
  - its backtrack limit is a separate per-search counter, not our budget;
  - it adds a dependency whose grammar we would still have to wrap to reject unprobed syntax.
- **One engine (backtracker) for everything.** Simpler, but backtracking is O(text × pattern) at best with no DFA
  scan. The lazy DFA path is what lets 100,000-cell columns fit the budget today. Rejected for regular patterns, kept
  for the empty-match retry only.
- **Backtracker only, behind a pre-check that sends "dangerous" patterns elsewhere.** No elsewhere exists for
  backreferences, so the split must be by feature (`is_regular`), not by danger.
- **Keep the `regex` crate's parser and pre-process the pattern string.** Rewriting `\Q..\E` and comments in text
  cannot give `a++`, `(?>..)` or backreferences a meaning, and two parsers would disagree on edge cases. One parser
  is directive 3.

## Test plan
- **Eval.** probes/regex-pcre2-features C1-C19, D1-D14 and B27-B29, t1626-regextest C29, t1626-regexreplace C34,
  regexreplace-limit-and-syntax A23, and the new probes/regex-pcre2-limits case.
- **Differential (unit, in `pcre/tests`).** For random regular patterns over a small alphabet (the property test style
  of T-1808), check:
  - parse, then `to_hir`, then `Engine` against parse, then `Program`, then `Backtracker`: same match and same
    capture spans;
  - our parser's `Hir` against `regex-syntax`'s for the patterns both dialects read alike (no `\Q`, no possessive,
    no POSIX classes). This catches translation slips.
- **Directive 5.**
  - Catastrophic patterns (`(a|a)*b`, `(a*)*b`, `((a+)+)+b`, nested lookarounds) on 32,767-character texts finish
    within the budget as `#ERROR!` or the probed answer, in bounded time.
  - A pattern of 250 nested groups and one of 32,767 characters parse without stack growth.
  - The cargo-fuzz `formula` target gains REGEX* calls with generated PCRE2 patterns (decision 0035's nightly fuzz).
- **Benchmarks.**
  - `crates/rustsheet/benches` gains backtracker cases (backreference over 10,000 cells, lookahead over long text,
    the empty-match retry over `a*?` on a 32,767-character text).
  - The T-1272 cases must not move by more than noise. Those patterns stay on the automata, and the only new cost
    is the `is_regular` check at compile time.

## Rollout
Each task is S/M and leaves `make check` green. A pattern needing an unimplemented feature stays `#VALUE!`, as today.

1. **Probe** (T-1972; research, sonnet, S): probes/regex-pcre2-limits with the "Unknowns" above. Its findings go
   into text.md.
2. **Parser and translation** (T-1973; functions-text, opus, M):
   - `pcre::parse`, `Ast`, `to_hir`;
   - `regex_functions.rs` builds through `Pattern`, with regular patterns only;
   - lands `\Q..\E`, comments, `(?x)`, scoped flags and Unicode POSIX classes (regexreplace-limit-and-syntax A23);
   - `a++` and the other backtracking-only constructs become `#VALUE!`, which is B29's direction (it is a failure
     today either way);
   - includes the differential test against `regex-syntax`.
3. **Backtracker core** (T-1900, rewritten; functions-text, opus, M):
   - `Program`, `Backtracker` and budget accounting;
   - backreferences, lookahead and lookbehind, atomic and possessive groups, `\K`, conditionals (C1-C19,
     B27-B29, t1626-regextest C29);
   - the differential test regular-on-automata against regular-on-backtracker.
4. **PCRE2 global iteration** (T-1899, rewritten; functions-text, opus, S): `find_iter` with the anchored not-empty
   retry on the backtracker and the `SplitSurrogate` rule (t1626-regexreplace C34, D1-D14).
5. **Limits and bench** (T-1974; perf, opus, S): fit `STEP`/`FRAME`, add the benchmarks and fuzz patterns, and apply
   the probe's match-limit answer (a `DEVIATION` entry where Excel answers past our budget).
6. **Unprobed features** found accepted by step 1 (recursion, subroutines, `\X`, `\R`, scripts) are filed as
   follow-up tasks by the step 1 task, one per feature group.

T-1829 (charging PikeVM iteration by measured work) is independent: it touches `regex_engine.rs` iteration costs,
which step 4 reuses unchanged.
