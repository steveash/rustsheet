---
id: T-2966
title: "Keep the run inside an empty call's parentheses (=NOW( ), =TODAY(  ); probes/t2786 I1, I11)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`=NOW( )` and `=TODAY(  )` print back with the run Excel keeps between the parentheses.

## Context
Split from T-2944, which kept the run of a missing argument after a comma (`Spaced` around `EmptyArgKind` in
`parser/lists.rs` `missing_argument`). A call with no arguments has no argument node to hold its run:
`parse_function_args_in_scope` returns `[]` at `(` `)`. Adding a `Spaced` empty argument would make every
arg-count check, and the stripped evaluation tree, see one argument. It needs its own home, for example a run
on the call node, or a lone spaced empty argument that `strip_spaces` and the count checks drop. Excel keeps
`=NOW( )` and `=TODAY(  )` (values as without the run) and Graph refuses `=SUM( )` (syntax.md "Whitespace
beside a missing argument is kept"; probes/t2786-whitespace-structured-refs-runs I1, I11).

## Acceptance criteria
- [x] I1 and I11 of probes/t2786-whitespace-structured-refs-runs pass, and NOW()/TODAY() still take no argument.
- [x] A probe answers whether Excel keeps a run before a missing first argument (`=SUM( ,1)`), and ours matches.
      The same probe confirms that `=SUBTOTAL(9,A1, )` and `=SUBTOTAL(9, ,A1)` are refused (test_subtotal_reference_args.rs
      H21, H22: hand-written, kept as main had them).
- [x] `make check` green; reviewers approve.

## Out of scope
- The runs beside a missing argument after a comma (T-2944, done).

## Outcome
`=NOW( )`, `=PI( )`, `=TRUE(` LF `)` and other empty built-in calls print back with their run: it is held as one
spaced empty argument (`Node::with_empty_call_run`) attached after the argument count is taken, and `strip_spaces`
removes it, so evaluation still sees no argument. A run before a missing first argument (`=SUM( ,1)`) is kept too.
New probes t2966-empty-call-runs and t2966-missing-first-arg-run (excel-graph) answer both. Typed
`=SUBTOTAL(9,C1, )` and `=SUBTOTAL(9, ,C1)` are refused (Graph 400). Eval delta: probes 2 fixed (t2786 I1, I11),
2 new cases passing, 0 regressions. Follow-up: T-2976 (empty LAMBDA and name calls).
