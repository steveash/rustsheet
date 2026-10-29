"""Unit tests for scripts/exit_owners.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import exit_owners  # noqa: E402 (the sys.path line above must run first)

GEN = "gen-m5/1d22a002d4f380b3.case.json"
TRIG = "ironcalc-calc/MATH/TRIGONOMETRIC.xlsx"
PROBE = "probes/index-reference-args.case.json"


def cell(sheet: str, a1: str, formula: str = "=X()") -> dict[str, object]:
    return {"sheet": sheet, "cell": a1, "formula": formula, "verdict": "numeric_ulp"}


def report() -> dict[str, object]:
    return {
        "milestone_exit": [
            {
                "milestone": "M5",
                "probe_row": "M5",
                "gen_visibly_off_cells": [f"{GEN}!Calc!A1", f"{GEN}!Calc!A107"],
            }
        ],
        "cases": [
            {
                "id": GEN,
                "suite": "gen-m5",
                # A2 fails within 15 digits: not visibly off, so not an exit cell (decision 0135).
                "cells": [cell("Calc", "A1"), cell("Calc", "A107", "=CSCH(PI()/4)"), cell("Calc", "A2")],
            },
            {"id": TRIG, "suite": "ironcalc-calc", "cells": [cell("S", "C51"), cell("S", "C55"), cell("S", "C60")]},
            {"id": PROBE, "suite": "probes", "cells": [cell("P", "B2"), cell("P", "B3"), cell("P", "B4")]},
        ],
        "function_scope": {
            "rows": [
                {
                    "milestone": "M5",
                    "failing": [f"{TRIG}!S!C51", f"{TRIG}!S!C55", f"{TRIG}!S!C60"],
                    "excepted": [f"{TRIG}!S!C55"],
                    "visibly_off": [f"{TRIG}!S!C51", f"{TRIG}!S!C55"],
                }
            ]
        },
        "probe_milestones": [
            {
                "milestone": "M5",
                "failing_cases": [[PROBE, 3]],
                "visibly_off_cells": [f"{PROBE}!P!B2", f"{PROBE}!P!B3"],
            }
        ],
    }


def criteria_cells(r: dict[str, object], milestone: str) -> list[tuple[str, str]]:
    """`(criterion, case!Sheet!A1)` of each exit cell of `milestone`, in order."""
    return [(c.criterion, f"{c.case}!{c.key}") for c in exit_owners.exit_cells(r, milestone)]


class TaskDir(unittest.TestCase):
    """A temporary tasks directory and a helper that writes a task into it."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.tasks = Path(self.tmp.name)
        self.n = 0

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def task(self, body: str, milestone: str = "M5", status: str = "todo") -> None:
        self.n += 1
        d = self.tasks / milestone
        d.mkdir(exist_ok=True)
        (d / f"T-{9000 + self.n}-x.md").write_text(
            f"---\nid: T-{9000 + self.n}\nmilestone: {milestone}\nstatus: {status}\n---\n\n{body}\n"
        )


class ExitOwners(TaskDir):
    def listed(self) -> list[str]:
        return [f"{c.case}!{c.key}" for c in exit_owners.unowned(report(), "M5", self.tasks)]

    def test_a_cell_named_by_an_open_task_is_owned(self) -> None:
        self.task("Owns 1d22a002d4f380b3 Calc!A107.")
        self.assertNotIn(f"{GEN}!Calc!A107", self.listed())
        self.assertIn(f"{GEN}!Calc!A1", self.listed())

    def test_done_or_other_milestone_tasks_own_nothing(self) -> None:
        self.task("1d22a002d4f380b3 A107", status="done")
        self.task("1d22a002d4f380b3 A107", milestone="M6")
        self.assertIn(f"{GEN}!Calc!A107", self.listed())

    def test_a_case_named_without_cells_owns_all_of_it(self) -> None:
        self.task("The failing cells of `probes/index-reference-args` are this task's.")
        listed = self.listed()
        self.assertNotIn(f"{PROBE}!P!B2", listed)
        self.assertNotIn(f"{PROBE}!P!B3", listed)

    def test_a_case_named_beside_other_cells_owns_only_those(self) -> None:
        self.task("- `TRIGONOMETRIC.xlsx` SINH_COSH_TANH!D51 and B15 (both pass now)")
        self.assertIn(f"{TRIG}!S!C51", self.listed())

    def test_an_excepted_calc_cell_is_not_listed(self) -> None:
        listed = self.listed()
        self.assertIn(f"{TRIG}!S!C51", listed)
        self.assertNotIn(f"{TRIG}!S!C55", listed)

    def test_only_visibly_off_cells_are_exit_cells(self) -> None:
        listed = self.listed()
        self.assertNotIn(f"{GEN}!Calc!A2", listed)
        self.assertNotIn(f"{TRIG}!S!C60", listed)
        self.assertNotIn(f"{PROBE}!P!B4", listed)

    def test_a_calc_row_shown_but_not_required_lists_no_cell(self) -> None:
        # M7's exit row shows the ironcalc-calc counts with a null `calc_met` (decision 0140).
        r = report()
        r["milestone_exit"][0].update({"calc_failing": 2, "calc_visibly_off": 1, "calc_met": None})
        listed = [f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M5", self.tasks)]
        self.assertNotIn(f"{TRIG}!S!C51", listed)
        self.assertIn(f"{PROBE}!P!B2", listed)
        r["milestone_exit"][0]["calc_met"] = False
        listed = [f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M5", self.tasks)]
        self.assertIn(f"{TRIG}!S!C51", listed)

    def test_m2_probe_cells_come_from_the_m0_m2_row(self) -> None:
        r = report()
        r["milestone_exit"] = [{"milestone": "M2", "probe_row": "M0-M2"}]
        r["function_scope"] = {"rows": []}
        r["probe_milestones"] = [
            {"milestone": "M0-M2", "failing_cases": [[PROBE, 3]], "visibly_off_cells": [f"{PROBE}!P!B3"]},
            {"milestone": "M5", "failing_cases": [[PROBE, 3]], "visibly_off_cells": [f"{PROBE}!P!B2"]},
        ]
        listed = [f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M2", self.tasks)]
        self.assertEqual(listed, [f"{PROBE}!P!B3"])
        self.task("probes/index-reference-args P!B3", milestone="M2")
        self.assertEqual(exit_owners.unowned(r, "M2", self.tasks), [])

    def test_m1_lists_gen_m1_smoke_and_the_m1_probes_list(self) -> None:
        # Decision 0146 item 1: two required suites and the frozen list as the probe measure.
        gen1, smoke = "gen-m1/abc.case.json", "smoke/basic.case.json"
        r = report()
        r["milestone_exit"] = [
            {
                "milestone": "M1",
                "gen_suite": "gen-m1",
                "gen_visibly_off_cells": [f"{gen1}!Calc!A3"],
                "more_suites": [{"name": "smoke", "visibly_off_cells": [f"{smoke}!S!B1"]}],
                "probe_row": "m1-probes",
            }
        ]
        r["cases"] = [
            {"id": gen1, "suite": "gen-m1", "cells": [cell("Calc", "A3")]},
            {"id": smoke, "suite": "smoke", "cells": [cell("S", "B1")]},
            {"id": PROBE, "suite": "probes", "cells": [cell("P", "B4", "=INDEX(A1:B2,1)")]},
        ]
        r["function_scope"] = {"rows": []}
        r["milestone_lists"] = [{"name": "m1-probes", "visibly_off_cells": [f"{PROBE}!P!B4"]}]
        cells = exit_owners.exit_cells(r, "M1")
        self.assertEqual(
            [(c.criterion, f"{c.case}!{c.key}", c.formula) for c in cells],
            [
                ("gen-m1", f"{gen1}!Calc!A3", "=X()"),
                ("smoke", f"{smoke}!S!B1", "=X()"),
                ("m1-probes", f"{PROBE}!P!B4", "=INDEX(A1:B2,1)"),
            ],
        )
        self.task("smoke/basic S!B1 and index-reference-args P!B4", milestone="M1")
        listed = [f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M1", self.tasks)]
        self.assertEqual(listed, [f"{gen1}!Calc!A3"])

    def test_m1_lists_a_failing_listed_case_with_no_cells_whole(self) -> None:
        # T-3091: a listed case that crashes has no cell; it prints as `case!*`, as in a probe row.
        r = report()
        r["milestone_exit"] = [{"milestone": "M1", "gen_suite": "", "probe_row": "m1-probes"}]
        r["cases"] = [{"id": PROBE, "suite": "probes", "cells": []}]
        r["function_scope"] = {"rows": []}
        r["milestone_lists"] = [{"name": "m1-probes", "failing_cases": [PROBE], "visibly_off_cells": []}]
        cells = exit_owners.exit_cells(r, "M1")
        self.assertEqual([(c.criterion, c.case, c.key) for c in cells], [("m1-probes", PROBE, exit_owners.WHOLE_CASE)])

    def test_m11_lists_corpus_bench_as_its_suite(self) -> None:
        corpus = "corpus-bench/53062/1_53062_input.xlsx"
        r = report()
        r["milestone_exit"] = [
            {"milestone": "M11", "gen_suite": "corpus-bench", "gen_visibly_off_cells": [f"{corpus}!Sheet1!D4"]}
        ]
        r["cases"] = [{"id": corpus, "suite": "corpus-bench", "cells": [cell("Sheet1", "D4")]}]
        r["function_scope"] = {"rows": []}
        r["probe_milestones"] = []
        self.assertEqual(criteria_cells(r, "M11"), [("corpus-bench", f"{corpus}!Sheet1!D4")])

    def test_m9_lists_calc_the_functions_below_the_rate_and_probes(self) -> None:
        # Decision 0146 item 3: no required suite; ironcalc-calc less exceptions, the
        # `attributed_off` cells of each function below the rate, and the M9 probe row.
        r = report()
        r["milestone_exit"] = [
            {
                "milestone": "M9",
                "gen_suite": "",
                "gen_visibly_off_cells": [],
                "calc_failing": 2,
                "calc_met": False,
                "below_rate_functions": ["IRR"],
                "probe_row": "M9",
            }
        ]
        r["function_scope"]["rows"][0]["milestone"] = "M9"  # type: ignore[index]
        r["probe_milestones"][0]["milestone"] = "M9"  # type: ignore[index]
        r["attributed_off"] = [
            {"case": GEN, "cell": "Calc!A1", "function": "IRR", "formula": "IRR(A1:A3)"},
            {"case": GEN, "cell": "Calc!A107", "function": "PMT", "formula": "PMT(1,2,3)"},
        ]
        self.assertEqual(
            criteria_cells(r, "M9"),
            [
                ("ironcalc-calc", f"{TRIG}!S!C51"),
                ("rate:IRR", f"{GEN}!Calc!A1"),
                ("probes", f"{PROBE}!P!B2"),
                ("probes", f"{PROBE}!P!B3"),
            ],
        )
        self.task("1d22a002d4f380b3 Calc!A1 is IRR's", milestone="M9")
        listed = [f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M9", self.tasks)]
        self.assertNotIn(f"{GEN}!Calc!A1", listed)
        self.assertIn(f"{TRIG}!S!C51", listed)

    def test_m8_lists_both_gen_suites_the_poi_subset_and_its_blocked_cases(self) -> None:
        # Decision 0146 item 2: gen-m8a, gen-m8b, the poi M8 subset (its visibly-off cells and,
        # as `case!*`, the cases holding a subset formula that did not run) and the M8 probes.
        gen_a, gen_b = "gen-m8a/aa.case.json", "gen-m8b/bb.case.json"
        poi, crashed = "poi/names.xls", "poi/tables.xlsx"
        r = report()
        r["milestone_exit"] = [
            {
                "milestone": "M8",
                "gen_suite": "gen-m8a",
                "gen_visibly_off_cells": [f"{gen_a}!S!A1"],
                "more_suites": [
                    {"name": "gen-m8b", "visibly_off_cells": [f"{gen_b}!S!B2"]},
                    {
                        "name": "poi M8 subset",
                        "visibly_off_cells": [f"{poi}!Data!C3"],
                        "blocked_cases": [f"{crashed}!*"],
                    },
                ],
                "probe_row": "M8",
            }
        ]
        r["cases"] = [
            {"id": gen_a, "suite": "gen-m8a", "cells": [cell("S", "A1")]},
            {"id": gen_b, "suite": "gen-m8b", "cells": [cell("S", "B2")]},
            {"id": poi, "suite": "poi", "cells": [cell("Data", "C3", "=SUM(Prices)")]},
            {"id": PROBE, "suite": "probes", "cells": [cell("P", "B2"), cell("P", "B3")]},
        ]
        r["function_scope"] = {"rows": []}
        r["probe_milestones"][0]["milestone"] = "M8"  # type: ignore[index]
        cells = exit_owners.exit_cells(r, "M8")
        self.assertEqual(
            [(c.criterion, f"{c.case}!{c.key}", c.formula) for c in cells],
            [
                ("gen-m8a", f"{gen_a}!S!A1", "=X()"),
                ("gen-m8b", f"{gen_b}!S!B2", "=X()"),
                ("poi M8 subset", f"{poi}!Data!C3", "=SUM(Prices)"),
                ("poi M8 subset", f"{crashed}!*", ""),
                ("probes", f"{PROBE}!P!B2", "=X()"),
                ("probes", f"{PROBE}!P!B3", "=X()"),
            ],
        )
        # A blocked case is owned by a task naming its stem (decision 0139).
        self.task("poi `tables.xlsx` crashes on import", milestone="M8")
        listed = [f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M8", self.tasks)]
        self.assertNotIn(f"{crashed}!*", listed)
        self.assertIn(f"{poi}!Data!C3", listed)

    def test_m10_lists_gen_m10_and_probe_text_items_beside_value_cells(self) -> None:
        # Decision 0146 item 4: gen-m10's visibly-off value cells and failing text items, and the
        # M10 probe row's visibly-off cells and failing text items.
        gen10, text_probe = "gen-m10/ab.case.json", "probes/text-locale.case.json"
        r = report()
        r["milestone_exit"] = [
            {
                "milestone": "M10",
                "gen_suite": "gen-m10",
                "gen_visibly_off_cells": [f"{gen10}!S!A1"],
                "gen_text_failing_cells": [f"{gen10}!S!B2"],
                "probe_text_items": 2,
                "calc_failing": None,
                "calc_met": None,
                "probe_row": "M10",
            }
        ]
        r["cases"] = [
            {"id": gen10, "suite": "gen-m10", "cells": [cell("S", "A1")]},
            {"id": PROBE, "suite": "probes", "cells": [cell("P", "B2")]},
            {"id": TRIG, "suite": "ironcalc-calc", "cells": [cell("S", "C51")]},
        ]
        # The report has an M10 function-scope row, but ironcalc-calc is no M10 criterion (a
        # null `calc_met`, decision 0146 item 4), so none of its cells is listed.
        r["function_scope"]["rows"][0]["milestone"] = "M10"  # type: ignore[index]
        r["probe_milestones"] = [
            {
                "milestone": "M10",
                "failing_cases": [[PROBE, 1]],
                "visibly_off_cells": [f"{PROBE}!P!B2"],
                "text_failing_cells": [f"{text_probe}!T!C4"],
            }
        ]
        self.assertEqual(
            criteria_cells(r, "M10"),
            [
                ("gen-m10", f"{gen10}!S!A1"),
                ("gen-m10 text", f"{gen10}!S!B2"),
                ("probes", f"{PROBE}!P!B2"),
                ("probes text", f"{text_probe}!T!C4"),
            ],
        )
        # A task naming the probe text item owns it; the others stay unowned.
        self.task("text-locale T!C4", milestone="M10")
        unowned = {f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M10", self.tasks)}
        self.assertEqual(unowned, {f"{gen10}!S!A1", f"{gen10}!S!B2", f"{PROBE}!P!B2"})
        # A row without the text criterion lists no probe text item.
        del r["milestone_exit"][0]["probe_text_items"]  # type: ignore[index]
        self.assertNotIn("probes text", [criterion for criterion, _ in criteria_cells(r, "M10")])

    def test_a1_does_not_match_a10(self) -> None:
        self.task("1d22a002d4f380b3 A107 and A10")
        listed = self.listed()
        self.assertIn(f"{GEN}!Calc!A1", listed)
        self.assertNotIn(f"{GEN}!Calc!A107", listed)

    def test_a_stem_must_be_whole(self) -> None:
        self.task("probes/index-reference-args-2")
        self.assertIn(f"{PROBE}!P!B2", self.listed())

    def test_a_stem_with_the_case_json_suffix_names_its_case(self) -> None:
        # The gap-triage `Cells:` line form (decision 0136).
        self.task("- gen-m5/1d22a002d4f380b3.case.json (1): Calc!A107")
        listed = self.listed()
        self.assertNotIn(f"{GEN}!Calc!A107", listed)
        self.assertIn(f"{GEN}!Calc!A1", listed)

    def test_a_longer_case_json_name_does_not_name_its_prefix(self) -> None:
        self.task("- gen-m5/1d22a002d4f380b3d.case.json (1): Calc!A107")
        self.task("- gen-m5/1d22a002d4f380b3.case.json.bak (1): Calc!A107")
        self.assertIn(f"{GEN}!Calc!A107", self.listed())

    def test_a_colon_span_owns_its_rectangle(self) -> None:
        self.task("probes/index-reference-args P!$C$2:B1")
        listed = self.listed()
        self.assertNotIn(f"{PROBE}!P!B2", listed)
        self.assertIn(f"{PROBE}!P!B3", listed)

    def test_a_colon_span_off_the_stem_line_owns_nothing(self) -> None:
        # A quoted formula's range is not a list of owned cells.
        self.task("probes/index-reference-args P!B3\n\n`=SUM(P!B1:B9)`")
        listed = self.listed()
        self.assertIn(f"{PROBE}!P!B2", listed)
        self.assertNotIn(f"{PROBE}!P!B3", listed)

    def test_a_hyphen_span_names_only_its_ends(self) -> None:
        self.task("index-reference-args B1-B3")
        listed = self.listed()
        self.assertIn(f"{PROBE}!P!B2", listed)
        self.assertNotIn(f"{PROBE}!P!B3", listed)

    def test_an_a1_name_beside_another_case_does_not_own(self) -> None:
        # The T-2425 leak (decision 0159): B2 is listed for the gen case, so the probe's B2 is not owned
        # although the same task names the probe on another line.
        self.task("- gen-m5/1d22a002d4f380b3: Calc!A107, B2\n- probes/index-reference-args: P!B3")
        listed = self.listed()
        self.assertIn(f"{PROBE}!P!B2", listed)
        self.assertNotIn(f"{PROBE}!P!B3", listed)
        self.assertNotIn(f"{GEN}!Calc!A107", listed)

    def test_an_indented_continuation_line_owns_beside_its_case(self) -> None:
        # The T-3031 form: a cell list that wraps onto indented lines of the same bullet.
        self.task("- probes/index-reference-args P!B3,\n  B2 and the rest")
        listed = self.listed()
        self.assertNotIn(f"{PROBE}!P!B2", listed)
        self.assertNotIn(f"{PROBE}!P!B3", listed)

    def test_a_new_line_bullet_row_or_blank_line_opens_a_block(self) -> None:
        for body in (
            "probes/index-reference-args P!B3\nB2",
            "- probes/index-reference-args P!B3\n  - B2",
            "- probes/index-reference-args P!B3\n  * B2",
            "- probes/index-reference-args P!B3\n  2. B2",
            "| probes/index-reference-args | P!B3 |\n| B2 |",
            "- probes/index-reference-args P!B3\n\n    B2",
        ):
            with self.subTest(body=body):
                for f in self.tasks.glob("*/*.md"):
                    f.unlink()
                self.task(body)
                listed = self.listed()
                self.assertIn(f"{PROBE}!P!B2", listed)
                self.assertNotIn(f"{PROBE}!P!B3", listed)

    def test_blocks(self) -> None:
        self.assertEqual(
            exit_owners.blocks("a\n  b\n- c\n  d\n\n  e\n| f"),
            ("a\n  b", "- c\n  d", "\n  e", "| f"),
        )

    def test_output_and_strict_exit(self) -> None:
        path = self.tasks / "report.json"
        path.write_text(json.dumps(report()))
        args = ["--milestone", "M5", "--report", str(path), "--tasks", str(self.tasks)]
        out = io.StringIO()
        with redirect_stdout(out):
            self.assertEqual(exit_owners.main(args), 0)
            self.assertEqual(exit_owners.main([*args, "--strict"]), 1)
        lines = out.getvalue().splitlines()
        self.assertIn(f"gen-m5\t{GEN}!Calc!A107\t=CSCH(PI()/4)", lines)
        self.assertEqual(lines[5], "5 failing M5 exit cells have no open M5 owner")

    def test_missing_report_or_milestone_fails_clearly(self) -> None:
        err = io.StringIO()
        with redirect_stderr(err):
            self.assertEqual(
                exit_owners.main(["--milestone", "M5", "--report", str(self.tasks / "no.json")]), 2
            )
            path = self.tasks / "report.json"
            path.write_text(json.dumps(report()))
            self.assertEqual(exit_owners.main(["--milestone", "M7", "--report", str(path)]), 2)
            self.assertEqual(exit_owners.main(["--milestone", "M5'", "--report", str(path)]), 2)
        self.assertIn("run `make eval` first", err.getvalue())
        self.assertIn("no milestone_exit row for M7", err.getvalue())


class SharedStems(TaskDir):
    """A stem several cases share names none of them (decision 0138)."""

    PV11 = "calipers/pv_goldens/pv26_011/golden.xlsx"
    PV12 = "calipers/pv_goldens/pv26_012/golden.xlsx"
    GEN9 = "gen-m9/146b46e74f2dac3b.case.json"
    COPY = "probes/t2436/146b46e74f2dac3b.case.json"
    CASES = (PV11, PV12, GEN9, COPY)

    def unowned(self) -> list[str]:
        r = {"unattributed_off": [{"case": c, "cell": "S!A1", "formula": "=X()"} for c in self.CASES]}
        return [f"{c.case}!{c.key}" for c in exit_owners.unowned_unattributed(r, self.tasks)]

    def test_case_names(self) -> None:
        called = exit_owners.case_names([*self.CASES, PROBE])
        self.assertEqual(called[PROBE], ("index-reference-args",))
        self.assertEqual(
            called[self.PV11],
            (self.PV11, "pv_goldens/pv26_011/golden.xlsx", "pv26_011/golden.xlsx", "pv26_011"),
        )
        # `t2436` is a directory of one case here, so it names that case too; a suite never does.
        self.assertEqual(called[self.COPY], ("probes/t2436/146b46e74f2dac3b", "t2436/146b46e74f2dac3b", "t2436"))
        self.assertEqual(called[self.GEN9], ("gen-m9/146b46e74f2dac3b",))

    def test_a_directory_two_cases_share_names_neither(self) -> None:
        other = "probes/t2436/0000000000000000.case.json"
        called = exit_owners.case_names([*self.CASES, other])
        self.assertEqual(called[self.COPY], ("probes/t2436/146b46e74f2dac3b", "t2436/146b46e74f2dac3b"))
        # Its stem is its own, so the shared directory does not matter.
        self.assertEqual(called[other], ("0000000000000000",))

    def test_a_calipers_directory_and_cell_own_only_that_workbook(self) -> None:
        self.task("Desktop-only cells: pv26_011 S!A1.", milestone="M14")
        self.assertEqual(self.unowned(), [f"{c}!S!A1" for c in (self.PV12, self.GEN9, self.COPY)])

    def test_a_gen_hash_does_not_own_its_probe_copy(self) -> None:
        self.task("- gen-m9/146b46e74f2dac3b.case.json (1): S!A1", milestone="M9")
        self.assertEqual(self.unowned(), [f"{c}!S!A1" for c in (self.PV11, self.PV12, self.COPY)])
        self.task("probes/t2436/146b46e74f2dac3b S!A1", milestone="M9")
        self.assertEqual(self.unowned(), [f"{self.PV11}!S!A1", f"{self.PV12}!S!A1"])

    def test_a_shared_stem_alone_names_no_case(self) -> None:
        self.task("Every failing cell of golden.xlsx and of 146b46e74f2dac3b.", milestone="M14")
        self.task("golden.xlsx S!A1, 146b46e74f2dac3b S!A1", milestone="M14")
        self.assertEqual(len(self.unowned()), 4)


class DirectoryPaths(TaskDir):
    """A case with its own stem is also named by its directory path (decision 0139)."""

    CB = "corpus-bench/53062/1_53062_input.xlsx"
    OTHER = "corpus-bench/5306/1_5306_input.xlsx"
    CASES = (CB, OTHER)

    def unowned(self) -> list[str]:
        r = {
            "unattributed_off": [
                {"case": c, "cell": f"S!{a1}", "formula": "=X()"} for c in self.CASES for a1 in ("AE6", "F6")
            ]
        }
        return [f"{c.case}!{c.key}" for c in exit_owners.unowned_unattributed(r, self.tasks)]

    def test_case_names(self) -> None:
        called = exit_owners.case_names([*self.CASES, PROBE])
        self.assertEqual(called[self.CB], ("1_53062_input.xlsx", "corpus-bench/53062"))
        self.assertEqual(called[PROBE], ("index-reference-args",))

    def test_a_path_two_cases_are_under_names_neither(self) -> None:
        nested = "corpus-bench/53062/sub/x.xlsx"
        self.assertEqual(exit_owners.case_names([self.CB, nested])[self.CB], ("1_53062_input.xlsx",))
        bare = "corpus-bench/53062.case.json"
        self.assertEqual(exit_owners.case_names([self.CB, bare])[self.CB], ("1_53062_input.xlsx",))

    def test_a_directory_path_alone_owns_the_whole_case(self) -> None:
        self.task("corpus-bench/53062 (`formamspnc (7)`): the case is a whole-case line.")
        self.assertEqual(self.unowned(), [f"{self.OTHER}!S!AE6", f"{self.OTHER}!S!F6"])

    def test_a_shorter_path_names_nothing(self) -> None:
        self.task("Every failing cell of corpus-bench/5306 and of corpus-bench/530.", milestone="M11")
        self.assertEqual(self.unowned(), [f"{self.CB}!S!AE6", f"{self.CB}!S!F6"])

    def test_a_directory_path_beside_a_cell_owns_only_that_cell(self) -> None:
        self.task("corpus-bench/53062 AE6")
        self.assertEqual(self.unowned(), [f"{self.CB}!S!F6", f"{self.OTHER}!S!AE6", f"{self.OTHER}!S!F6"])

    def test_the_full_id_still_names_the_case(self) -> None:
        self.task("`corpus-bench/53062/1_53062_input.xlsx` F6")
        self.assertEqual(self.unowned(), [f"{self.CB}!S!AE6", f"{self.OTHER}!S!AE6", f"{self.OTHER}!S!F6"])

    def test_a_name_may_end_a_sentence(self) -> None:
        self.task("The spill cases of corpus-bench/5306.")
        self.assertEqual(self.unowned(), [f"{self.CB}!S!AE6", f"{self.CB}!S!F6"])


class UnattributedOwners(TaskDir):
    UNATTRIBUTED = {
        "unattributed_off": [
            {
                "case": PROBE,
                "cell": "P!B2",
                "formula": "GAMMA(BETA.DIST(0.5,2,3,TRUE))",
            },
            {"case": GEN, "cell": "Calc!A9", "formula": "IF(AND(A1,B1),1,2)"},
        ]
    }

    def unowned(self) -> list[str]:
        cells = exit_owners.unowned_unattributed(self.UNATTRIBUTED, self.tasks)
        return [f"{c.case}!{c.key}" for c in cells]

    def test_an_unattributed_cell_is_listed_until_an_open_task_of_any_milestone_owns_it(
        self,
    ) -> None:
        self.assertEqual(self.unowned(), [f"{PROBE}!P!B2", f"{GEN}!Calc!A9"])
        self.task("index-reference-args B2", status="done")
        self.assertIn(f"{PROBE}!P!B2", self.unowned())
        self.task("index-reference-args B2", milestone="M9")
        self.assertEqual(self.unowned(), [f"{GEN}!Calc!A9"])

    def test_unattributed_output_and_strict_exit(self) -> None:
        path = self.tasks / "report.json"
        path.write_text(json.dumps(self.UNATTRIBUTED))
        args = ["--unattributed", "--report", str(path), "--tasks", str(self.tasks)]
        out = io.StringIO()
        with redirect_stdout(out):
            self.assertEqual(exit_owners.main(args), 0)
            self.assertEqual(exit_owners.main([*args, "--strict"]), 1)
        lines = out.getvalue().splitlines()
        self.assertEqual(lines[0], f"{PROBE}!P!B2\tGAMMA(BETA.DIST(0.5,2,3,TRUE))")
        count = "2 visibly-off cells attributed to no function have no open owner"
        self.assertEqual(lines[2], count)
        # A report from before T-0082 has no list: nothing is unowned.
        path.write_text("{}")
        with redirect_stdout(io.StringIO()):
            self.assertEqual(exit_owners.main([*args, "--strict"]), 0)


class AllOff(TaskDir):
    """`--all-off` (T-0086): the visibly-off rule is the eval's (closeness.rs pins the 4-ulp and
    half-unit boundaries); the report's lists arrive already filtered."""

    @staticmethod
    def report() -> dict[str, object]:
        return {
            "cases": [
                {"id": PROBE, "suite": "probes", "cells": [cell("P", "B2"), cell("P", "B3"), cell("P", "B4")]}
            ],
            "attributed_off": [
                {"case": PROBE, "cell": "P!B3", "function": "GAMMA", "formula": "GAMMA(1)",
                 "expected": "1.0", "actual": "2.0"},
                {"case": PROBE, "cell": "P!B2", "function": "BETA.DIST", "formula": "BETA.DIST(1)",
                 "expected": "1.0", "actual": "2.0"},
            ],
            "unattributed_off": [{"case": PROBE, "cell": "P!B4", "formula": "A()+B()"}],
            "probe_milestones": [
                {"milestone": "M9", "failing_cases": [[PROBE, 3]],
                 "visibly_off_cells": [f"{PROBE}!P!B3", f"{PROBE}!P!B5"]},
            ],
        }

    def listed(self) -> list[tuple[str, str, bool]]:
        return [
            (o.cell.criterion, o.cell.key, deferred)
            for o, deferred in exit_owners.unowned_off(self.report(), self.tasks)
        ]

    def test_groups_by_function_then_unattributed_then_probe(self) -> None:
        self.assertEqual(
            self.listed(),
            [
                ("BETA.DIST", "P!B2", False),
                ("GAMMA", "P!B3", False),
                ("unattributed", "P!B4", False),
                ("probe M9", "P!B5", False),
            ],
        )

    def test_an_open_task_owns_its_cell_and_a_deferred_one_does_not(self) -> None:
        self.task("index-reference-args B3")
        self.assertNotIn("P!B3", [k for _, k, _ in self.listed()])
        d = self.tasks / "M5" / "T-9100-d.md"
        d.write_text(
            "---\nid: T-9100\nmilestone: M5\nstatus: todo\ndefer: phase-1b\n---\n\nindex-reference-args B2\n"
        )
        self.assertIn(("BETA.DIST", "P!B2", True), self.listed())

    def test_output_function_filter_and_strict(self) -> None:
        path = self.tasks / "report.json"
        path.write_text(json.dumps(self.report()))
        args = ["--all-off", "--report", str(path), "--tasks", str(self.tasks)]
        out = io.StringIO()
        with redirect_stdout(out):
            self.assertEqual(exit_owners.main([*args, "--function", "GAMMA", "--strict"]), 1)
        lines = out.getvalue().splitlines()
        self.assertEqual(lines[0], f"GAMMA\t{PROBE}!P!B3\tGAMMA(1)\t1.0\t2.0")
        self.assertEqual(lines[-1], "1 visibly-off cells have no open owner")


if __name__ == "__main__":
    unittest.main()
