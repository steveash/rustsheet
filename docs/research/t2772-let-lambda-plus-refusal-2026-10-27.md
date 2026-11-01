# T-2772: typed LET name and LAMBDA parameter under a unary plus (excel-graph, 2026-10-27)

## Question

Does Excel accept a unary plus before a LET name (`=LET(+x,1,x)`) or a LAMBDA parameter (`=LAMBDA(+x,x)(1)`)?
Excel refuses both, and a refused formula cannot be committed as an oracle-filled case. This note is the record
that syntax.md and T-2887 cite.

## Method

`rustsheet-oracle fill -b excel-graph --suite probes` was run on two one-sheet spec cases. Each case had
`S!A1 = {"entered": <formula>}` and `S!B1 = =FORMULATEXT(A1)`. The backend uploads the workbook, then writes A1 with
a range `values` PATCH (typed input). The cases were deleted after the run because a refused input leaves no sidecar.

## Results

| typed formula | PATCH response | Graph request-id |
|---|---|---|
| `=LAMBDA(+x,x)(1)` | 400 `InvalidArgument`, "The argument is invalid or missing or has an incorrect format." (2026-10-27T09:35:01Z) | d126173b-ea96-4940-a857-b8426b75dcae |
| `=LET(+x,1,x)` | 400 `InvalidArgument`, same message (2026-10-27T09:35:07Z) | 9c0c92f4-291c-4faf-b632-ab245dcb6d6d |

This is the same response Excel gives for every typed formula it refuses (syntax.md, argument counts).

## Ours

- `=LAMBDA(+x,x)(1)` gives `#ERROR!`, our refusal value, pinned by test_fn_let.rs.
- `=LET(+x,1,x)` gives `#VALUE!`: LET's evaluator reads a declaration that is not a name as a run-time `#VALUE!`. Filed as T-2887.
