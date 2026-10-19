---
id: T-1898
title: "Spec: a PCRE2-compatible backtracking matcher for the REGEX functions"
milestone: M7
lane: functions-text
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ decides how RustSheet matches Excel's PCRE2 regular expressions: the engine (a hand-written backtracking matcher, or a crate), how it is bounded under directive 5, and how it shares work with the current lazy-DFA engine.

## Context
Found by T-1688. Excel's REGEX* functions are PCRE2: probes/regex-pcre2-features shows backreferences (`\1`, `\g1`, `\k<n>`, case-insensitive `(a)\1` matching `Aa`), lookahead and fixed-length lookbehind (`(?<=a+)` is `#VALUE!`), possessive and atomic groups, `\K`, conditionals `(?(1)b|c)`, comments, `\Q..\E`, `(?x)` and scoped inline flags `(?-i)`, all of which the `regex` crate rejects or lacks; and PCRE2's global iteration (after an empty match, a non-empty match anchored at the same position is tried first: `a*?` on `aaa` replaces to `bbbbbbb`, `|b` on `abc` to `-a---c-`; D1-D12, t1626-regexreplace C34), which Rust's leftmost-first iterator cannot express because it has no "not empty at start" search. Pattern errors are `#VALUE!` and a possessive pattern that cannot match is `#N/A` (B27-B29). The current engine (functions/text/regex_engine.rs) counts every search's work (DEVIATION, decision 0014); a backtracking matcher needs a step budget and recursion-free design (directive 5), and Excel's own limits (PCRE2 match and depth limits; what `(a|a)*b` on a long text gives) should be probed. Consider: one engine for everything versus the backtracker only for patterns or iterations that need it; Unicode classes (`[[:alpha:]]` is Unicode in Excel, T-1817); UTF-16 versus UTF-8 positions.

## Acceptance criteria
- [x] docs/specs/ has the spec, with a decision record if it adds a dependency.
- [x] The implementation is split into S/M tasks (T-1899 and T-1900 are rewritten or replaced to follow it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the matcher.

## Outcome
SPEC-0013 (docs/specs/SPEC-0013-pcre2-regex-matcher.md) is accepted. Its design:
- One hand-written, iterative PCRE2 parser for every pattern.
- Regular patterns are translated to `regex_syntax::hir::Hir` and stay on the existing lazy-DFA/PikeVM `Engine`, with
  the same costs.
- Every other pattern runs on a recursion-free backtracking VM. Its steps and frames are charged to the call's
  existing budget.
- Iteration is PCRE2's: an empty match is followed by an anchored not-empty retry on the backtracker, and the search
  advances one UTF-16 unit, `#VALUE!` inside a surrogate pair.
- No new crate. `regex-syntax`, already locked, becomes a direct dependency in T-1973.
- `pcre2` (C FFI, wasm32, unsafe) and `fancy-regex` (Rust dialect and iteration, separate limit) were rejected.

Split, in order:
1. T-1972, probe (research, sonnet), which now also holds T-1903's per-class POSIX probe;
2. T-1973, parser (opus, M; depends on the T-1830 benchmarks), then T-1903 (rewritten, S: POSIX classes in the new parser);
3. T-1900, backtracker (rewritten; depends on T-1973);
4. T-1899, iteration (rewritten, S; depends on T-1900);
5. T-1974, costs, benchmarks and fuzz (perf).

No eval change (docs only).

## Blocked because
The review hit its 3-round limit (work-loop step 5). One verdict is missing: the performance reviewer's final
re-review.

Verdicts on head 30135095 and later:
| reviewer | verdict |
|---|---|
| security | APPROVE (at c17d37cc) |
| architecture | APPROVE (round 1, diff unchanged in scope since) |
| quality | APPROVE (at 30135095) |
| performance | REQUEST_CHANGES (round 3) |

Performance round 3 found one correctness rule missing from the memoized mode: a successful match must invalidate the
visited marks at or before its end. Security raised the same point as a minor finding. The rule is now in SPEC-0013
section 3 and T-1899, and T-1974 holds the memoized-mode DEVIATION.

To unblock:
1. Run the performance reviewer (and quality and architecture, since the spec changed after their approval) on
   `git diff origin/main...HEAD` of this branch (PR #1167).
2. If all approve, set `status: done`, tick the box, and merge.

