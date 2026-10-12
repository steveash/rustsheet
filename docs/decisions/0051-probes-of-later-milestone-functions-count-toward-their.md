# 0051 — Probes of later-milestone functions count toward their own milestone, not M2

- **Status:** accepted (2026-10-12, owner answer to decision 0043's open question; measurement in
  T-1423). Amends decision 0043, item 2.
- **Context:**
  - Decision 0043 left an open owner question: should probes of functions whose milestone is
    after M2 count toward M2 exit, or toward each function's own milestone? Until the owner
    answered, they counted for M2.
  - The owner (2026-10-12): do not block M2 on future-milestone probes. They count toward the
    function's own milestone.
  - At origin/main 9ad32ae1, eval/baseline/probes.txt has 2,881 F lines. Most are probes of
    later-milestone functions. The biggest cases, with their case.json tags:
    - `groupby-pivotby` 1,431 (`M7`);
    - `stat-argument-modes` 403, `fin-argument-modes` 306, `eng-argument-modes` 119 and
      `db-argument-modes` 38 (all `M9`);
    - `m7-argument-modes` 114 (`M7`);
    - `date-argument-modes` 95 and `lookup-argument-modes` 74 (`M6`);
    - `math-argument-modes` 9 (`M5`).
    These tags match their subjects, so they need no override.
  - Other cases carry a tag that does not match their subject. Before decision 0043, `M1` was the
    usual tag for any probe, so GEOMEAN, XIRR, WEEKNUM, MOD and TEXT-format probes are tagged
    `M1`. The catalog availability probes (`probes/catalog/*`) are tagged `M0`, although each of
    their cells calls a different function. Five cases are tagged `M9` but probe M6, M7 or M8
    subjects. Case files are never edited to fix a tag, because that could change oracle input
    hashes (decision 0043).
- **Decision:**
  1. **Every probe case has one milestone, deterministically.** The first rule that applies, in
     this order, decides:
     1. A case on the frozen list `eval/milestones/m1-probes.txt` is `M1`. The list is
        unchanged (decision 0043).
     2. A case listed in `eval/milestones/probe-milestones.tsv` takes the milestone given there.
        The file holds tab-separated lines: case id, milestone, subject. It has 142 lines.
     3. In `probes/catalog/*`, the milestone is set per checked cell. A cell takes the highest
        catalog milestone (`eval/catalog/scope.json`) of the functions its formula calls,
        written `NAME(` (an `_xlfn.` prefix does not change the name), apart from the observing
        wrappers `OR`, `IFERROR`, `ERROR.TYPE` and `INDEX`. A cell that calls no catalog
        function is `M2`. Each availability cell calls exactly one function, so the cell counts
        toward that function's milestone; the one exception is `availability-13` A2, which tests
        ERROR.TYPE itself, so dropping the wrapper leaves it `M2`.
     4. Any other case takes its milestone tag (`M0`, `M1`, `M2`, `M5`…`M11`) from its
        case.json `tags`. No case has two milestone tags today. If one ever has, the highest
        wins.
     5. A case with no milestone tag is `M2`, as before. Today there are two:
        `geomean-same-product`, which is overridden to M9, and `not-array-cast`.
     "Highest" follows the phase order of docs/milestones.md. For M2 and M5–M11, that is
     numeric order.
  2. **When a case gets an override line.** An override line is added when a case's tag does not
     match its subject. The subject is what the case id and description say it probes. Functions
     that only observe the result (ERROR.TYPE, ISREF, ROWS, FORMULATEXT, wrapping IF/IFERROR) do
     not count. The line gives the highest catalog milestone among the subject functions, or the
     milestone of a subject feature: 3-D and external references and defined-name bodies are M8,
     and number-format codes are M10.
     - Features are included. The owner's answer names functions, but 3-D references and number
       formats are just as much "future-milestone probes" (M8 and M10 scope in
       docs/milestones.md). Decision 0043 already called them M8 and M10 subjects.
     - **Exception:** a case tagged `M0`, `M1` or `M2` (or untagged) whose subject includes an
       M2 function or an M1 subject keeps its tag. This way M2 never loses evidence about its
       own functions. Examples that stay M2:
       - `volatile-invariant` (NOW and TODAY);
       - `aggregate-error-precedence`;
       - `array-context-legacy-sumproduct` (its SUM(IF()) control and INDEX);
       - `text-surrogate-positions` (LEN, LEFT, RIGHT, MID);
       - `date-system-1904-negatives` and `date-system-1900-negatives` (YEAR);
       - `subtotal-reference-operands-c` and `-d` (SUM, MIN, MAX, COUNT);
       - the criteria-grammar cases (`criteria-*`, `ifs-*`, `date-text-dotted-criteria`), since
         the criteria grammar is M2 scope;
       - `clock-text-*` and `datevalue-comma-spaces` (coercion grammar);
       - `subnormal-*` apart from `subnormal-function-boundaries`.
       A later-tagged case whose tag is wrong is overridden either way. For example,
       `filter-if-empty-array` (tagged `M9`) is `M7`.
     - The 142 lines, by milestone:
       - **M5 (21):** AGGREGATE `aggregate-args`, `-array-args`, `-grid`, `-range-args`;
         `subtotal-hidden-filtered`; `ceiling-xlfn-prefix`;
         `gcd-lcm-bounds`, `lcm-result-bound`, `lcm-result-edge`; `ifs-switch-array-values`,
         `ifs-switch-lift-edges`; `is-functions-array-lift`; `mod-*` (8);
         `text-b-functions-broadcast`.
       - **M6 (22):** `subtotal-reference-operands`, `-b`, `-e`, `-f` (AREAS); `weeknum-*` (4);
         `edate-serial-0-months`; `lookup-2d-vector-wide`,
         `lookup-vector-edges`; `xlookup-*` and `xmatch-*` (8); `xlookup-row-column-picks`,
         `offset-negative-height-width` and `indirect-text-broadcast` (all three tagged `M9`,
         the first counted in `xlookup-*`); `rows-columns-args`.
       - **M7 (15):** `let-*` (13), `lambda-param-call`, and `filter-if-empty-array` (tagged
         `M9`).
       - **M8 (12):** `3d-ref-*` (9), `external-ref-no-link`, `defined-name-formula-bodies`, and
         `isref-names-filter-if-empty` (tagged `M9`; defined names and FILTER).
       - **M9 (46):** `geomean-*` (16); `xirr-*` (3), `xnpv-argument-count` and
         `whole-column-empty-walkers`; `variance-*` (7); `rank-*` (10); `large-small-k`;
         `dvar-one-vs-two-pass`; `countblank-*` (5); `subnormal-function-boundaries` (ERFC and
         the normal distribution).
       - **M10 (26):** TEXT format-code cases: `elapsed-*` (17), `negative-date-sections-1900`
         and `-1904`, `text-date-255-char-limit`, `text-format-length-limit` and `-b`,
         `text-ddd-negative-fraction`, `text-general-exponent-boundary`,
         `text-negative-serial-elapsed-mixed`; and `dates-early-1900-edges` (ddd/dddd of serial
         0, next to WEEKNUM and EDATE).
     - New probe cases carry the milestone tag of the function or feature they probe, so they
       need no line. The file grows only when an existing tag turns out wrong. It is under
       `eval/milestones/`, so any edit needs a decision record (decision 0044). The harness reads
       only `*.txt` lists there, so this file does not become a frozen list.
  3. **Exit criteria.**
     - M2 exit requires every probe case, and every catalog cell, whose milestone is M0, M1 or M2
       to be at 100% bit-exact, excluding accepted deviations (decision 0036). The M0 cases (such
       as `resource-limits`, `lifted-range-*` and `rich-errors-*`) and the M1-tagged cases that
       are not on the frozen list still count for M2, as in decision 0043.
     - M5, M6, M7, M8, M9, M10 and M11 each gain "its probe cases at 100% bit-exact, excluding
       accepted deviations".
     - M1's frozen list criterion does not change.
     - This replaces decision 0043 item 2's "the whole `probes` suite" for M2.
  4. **Fix before discovering more, per milestone.** Decision 0043 item 3 applies to the current
     milestone's assigned failures. The steward picks the Critical lane from the failures
     assigned to M0–M2, not from the whole suite's.
- **Measurement** (origin/main 9ad32ae1, eval/baseline/probes.txt, 2,881 F lines):

  | Assigned milestone | F lines |
  |---|---|
  | M0, M1, M2 (frozen M1 list: 0) | 74 |
  | M5 | 35 |
  | M6 | 175 |
  | M7 | 1,547 |
  | M8 | 3 |
  | M9 | 994 |
  | M10 | 53 |
  | M11 | 0 |

  - The 74 M0–M2 F lines are in 12 cases:
    - `aggregate-error-precedence` 16 and `volatile-invariant` 16;
    - `array-context-legacy-sumproduct` 12;
    - `date-system-1904-negatives` 8;
    - `text-surrogate-positions` 6;
    - `concatenate-limit-spill` 4 and `counta-empty-value` 4;
    - `lookup-pick-index-shapes` 3;
    - `written-parens-reference-only` 2;
    - `rich-errors-image-sizing` 1, `subtotal-reference-operands-d` 1 (a GEOMEAN cell) and
      `text-order-nbsp-space` 1.
  - Known consequences of the per-case rule (fidelity review):
    - `timevalue-*` (4), `clock-text-*` and `near-zero-aggregate-forms` keep their M0/M1/M2 tags:
      half their cells are operator coercion of time text (`0+A`) or near-zero snapping, which
      are M1/M2 subjects.
    - `subtotal-reference-operands-d` stays M2 for its SUBTOTAL reference-operand cells, so its
      one F line, the M9 GEOMEAN cell H28 (`=GEOMEAN(5,7)`), still counts toward M2 exit. T-1423
      adds per-cell overrides to the mapping file so that cell can move to M9.
    - Deliberate keeps with no F line today: `name-body-length-*` (7, name-body limits that
      agents and the xlsx reader share), `subnormal-sumsq-sumproduct-edges` and
      `ifs-shape-mismatch` (the shared criteria engine). Revisit if they gain failures.
  - Catalog cells: the 503 availability cells split M2 38, M5 131, M6 39, M7 32 and M9 263. Their
    6 F lines are:
    - `availability-03` A3, BAHTTEXT (M5);
    - `availability-14` A7, FILTERXML (M5);
    - `availability-17` A3, GETPIVOTDATA (M6);
    - `availability-17` A4, GROUPBY (M7);
    - `availability-30` A3, PIVOTBY (M7);
    - `function-name-forms` A7, FILTERXML (M5).
  - Until T-1423 lands, the steward counts the F lines from the repository root with this
    command. It prints one count per assigned milestone:

    ```
    python3 -c 'import json,re,collections as C;S={f["name"]:f["milestone"] for f in json.load(open("eval/catalog/scope.json"))["functions"]};L={l.strip() for l in open("eval/milestones/m1-probes.txt") if l.strip() and l[0]!="#"};O=dict(l.split("\t")[:2] for l in open("eval/milestones/probe-milestones.tsv") if l[0]!="#");J=lambda c:json.load(open("eval/suites/probes/cases/"+c[7:]));K=lambda c,i:max([S[n] for n in re.findall(r"([A-Z][A-Z0-9.]*)\(",str({s["name"]+"!"+a:v for s in J(c)["sheets"] for a,v in s["cells"].items()}.get(i,""))) if n in S and n not in ("OR","IFERROR","ERROR.TYPE","INDEX")],key=lambda m:int(m[1:]),default="M2");T=lambda c:([x for x in J(c).get("tags",[]) if re.fullmatch(r"M\d+",x)]+["M2"])[0];M=lambda c,i:"M1" if c in L else O[c] if c in O else K(c,i) if c.startswith("probes/catalog/") else T(c);N=C.Counter(("M0-M2" if m in ("M0","M1","M2") else m) for m in (M(p[1],p[2]) for p in (l.rstrip("\n").split("\t") for l in open("eval/baseline/probes.txt")) if p[0]=="F"));print(" ".join(f"{k}={N[k]}" for k in ["M0-M2"]+[f"M{n}" for n in range(5,12)]),"total="+str(sum(N.values())))'
    ```

    At 9ad32ae1 it prints
    `M0-M2=74 M5=35 M6=175 M7=1547 M8=3 M9=994 M10=53 M11=0 total=2881`. M1's frozen list is
    still read from the report's "Milestone probe lists" section (decision 0044).
- **Consequences:**
  - docs/milestones.md: the M2 exit criterion cites this record, and M5–M11 each gain the probe
    criterion. `.claude/skills/steward/SKILL.md` ("Frozen probe lists") reports the M0–M2 count,
    not the whole suite's, when it chooses the Critical lane.
  - T-1423 (harness, M2) makes `make eval` report scored cells, bit-exact cells, deviations and
    known failures for each assigned milestone, using rules 1–5 above. T-1146's per-list report
    stays as it is.
  - M2's finish line no longer moves with later-milestone probes: 74 F lines today, not 2,881.
    GEOMEAN, XIRR, WEEKNUM, MOD, TEXT-format and catalog availability failures now count toward
    M9, M9, M6, M5, M10 and their functions' milestones. Open tasks keep their ids and milestone
    labels. A label does not change which exit a case counts toward.
