"""Unit tests for scripts/tasks.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasks  # noqa: E402 (the sys.path line above must run first)


def task(tid: str, milestone: str) -> dict[str, object]:
    return {"id": tid, "milestone": milestone}


class NextId(unittest.TestCase):
    def test_takes_next_after_highest_in_the_milestones_block(self) -> None:
        ts = [task("T-0004", "M0"), task("T-0023", "M0"), task("T-0300", "M0"), task("T-0110", "M1")]
        taken = {str(t["id"]) for t in ts}
        self.assertEqual(tasks.next_id("M0", ts, taken), "T-0024")
        self.assertEqual(tasks.next_id("M1", ts, taken), "T-0111")

    def test_never_reuses_a_gap(self) -> None:
        ts = [task("T-0002", "M0"), task("T-0009", "M0")]
        self.assertEqual(tasks.next_id("M0", ts, {"T-0002", "T-0009"}), "T-0010")

    def test_ids_on_other_branches_count_as_taken(self) -> None:
        ts = [task("T-0012", "M0")]
        # T-0013 and T-0014 exist only on another session's pushed branch.
        self.assertEqual(tasks.next_id("M0", ts, {"T-0012", "T-0013", "T-0014"}), "T-0015")

    def test_new_milestone_or_full_block_goes_above_everything(self) -> None:
        ts = [task("T-0099", "M0"), task("T-0932", "M9")]
        taken = {"T-0099", "T-0932"}
        self.assertEqual(tasks.next_id("M0", ts, taken), "T-0933")
        self.assertEqual(tasks.next_id("M5", ts, taken), "T-0933")


class GitTaskIds(unittest.TestCase):
    def test_reads_task_files_on_every_remote_branch(self) -> None:
        outputs = {
            ("for-each-ref",): "refs/remotes/origin/main\nrefs/remotes/origin/agent/x\n",
            ("ls-tree", "refs/remotes/origin/main"): "docs/tasks/M0/T-0010-a.md\ndocs/tasks/BOARD.md\n",
            ("ls-tree", "refs/remotes/origin/agent/x"): "docs/tasks/M0/T-0013-b.md\n",
        }

        def fake_run(cmd: list[str], **_: object) -> mock.Mock:
            args = cmd[3:]
            key = ("for-each-ref",) if args[0] == "for-each-ref" else ("ls-tree", args[3])
            return mock.Mock(stdout=outputs[key])

        with mock.patch.object(tasks.subprocess, "run", side_effect=fake_run):
            self.assertEqual(tasks.git_task_ids(fetch=False), {"T-0010", "T-0013"})

    def test_without_git_falls_back_to_nothing(self) -> None:
        with mock.patch.object(tasks.subprocess, "run", side_effect=OSError("no git")):
            self.assertEqual(tasks.git_task_ids(fetch=False), set())


class NewTask(unittest.TestCase):
    def run_new(self, *argv: str) -> tuple[int, list[Path]]:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root), mock.patch.object(tasks, "TASKS", root / "docs/tasks"), \
                    mock.patch.object(tasks.sys, "argv", ["tasks.py", "new", *argv]), \
                    mock.patch.object(tasks, "git_task_ids", return_value={"T-0003"}):
                code = tasks.new_task([task("T-0002", "M0")], fetch=False)
                return code, sorted(p.relative_to(root) for p in root.rglob("*.md"))

    def test_writes_the_next_free_id_from_the_template(self) -> None:
        code, files = self.run_new("--milestone", "M0", "--lane", "harness", "--title", "Do a thing")
        self.assertEqual(code, 0)
        self.assertEqual(files, [Path("docs/tasks/M0/T-0004-do-a-thing.md")])

    def test_tier_is_written_and_validated(self) -> None:
        code, files = self.run_new("--milestone", "M0", "--lane", "harness", "--title", "t", "--tier", "sonnet")
        self.assertEqual((code, len(files)), (0, 1))
        code, files = self.run_new("--milestone", "M0", "--lane", "harness", "--title", "t", "--tier", "haiku")
        self.assertEqual((code, files), (2, []))

    def test_title_with_quotes_and_backslashes_round_trips(self) -> None:
        title = 'TEXT(x,"0;-0") is \\ ";;;" `$x`'
        text = tasks.TEMPLATE.format(id="T-0004", title=tasks.quote(title), milestone="M0", lane="harness",
                                     priority="P2", size="S", tier="opus")
        self.assertIn('title: "TEXT(x,\\"0;-0\\") is \\\\ \\";;;\\" `$x`"', text)
        self.assertEqual(tasks.front_matter(text, Path("t"))["title"], title)
        self.assertEqual(tasks.unquote("plain"), "plain")

    def test_rejects_bad_arguments(self) -> None:
        for argv in (("--milestone", "M0x", "--lane", "harness", "--title", "x"),
                     ("--milestone", "M0", "--lane", "nope", "--title", "x"),
                     ("--milestone", "M0", "--lane", "harness", "--title", "a\nstatus: done"),
                     ("--milestone", "M0", "--lane", "harness", "--title", "x", "--size", "L")):
            code, files = self.run_new(*argv)
            self.assertEqual((code, files), (2, []), argv)


def write_task(
    root: Path,
    tid: str,
    status: str = "todo",
    tier: str | None = "opus",
    body: str = "",
    lane: str = "harness",
    priority: str = "P1",
    milestone: str = "M0",
    depends_on: str = "",
    campaign: str | None = None,
    defer: str | None = None,
) -> Path:
    path = root / "docs/tasks" / milestone / f"{tid}-x.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    tier_line = f"tier: {tier}\n" if tier else ""
    campaign_line = f"campaign: {campaign}\n" if campaign else ""
    campaign_line += f"defer: {defer}\n" if defer else ""
    path.write_text(
        f"---\nid: {tid}\ntitle: \"x\"\nmilestone: {milestone}\nlane: {lane}\nstatus: {status}\npriority: {priority}\n"
        f"size: S\n{tier_line}{campaign_line}depends_on: [{depends_on}]\n---\n{body}"
    )
    return path


def run_next(
    root: Path,
    tier: str,
    claims: tuple[set[str], set[str]] = (set(), set()),
    active: frozenset[tuple[str, str]] = frozenset(),
    lane: str | None = None,
    limited: dict[str, str] | None = None,
) -> str:
    return run_next_line(root, tier, claims, active, lane, limited).split()[0]


def run_next_line(
    root: Path,
    tier: str,
    claims: tuple[set[str], set[str]] = (set(), set()),
    active: frozenset[tuple[str, str]] = frozenset(),
    lane: str | None = None,
    limited: dict[str, str] | None = None,
) -> str:
    argv = ["tasks.py", "next", "--tier", tier, *(["--lane", lane] if lane else [])]
    with mock.patch.object(tasks, "ROOT", root), mock.patch.object(tasks, "TASKS", root / "docs/tasks"), \
            mock.patch.object(tasks.sys, "argv", argv), \
            mock.patch.object(tasks, "branch_claims", return_value=(*claims, set(active), limited or {})), \
            mock.patch("builtins.print") as printed:
        tasks.main()
    return str(printed.call_args[0][0])


class Tiers(unittest.TestCase):
    def test_open_tasks_need_a_valid_tier_done_ones_do_not(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(write_task(root, "T-0001", tier=None)),
                    tasks.parse(write_task(root, "T-0002", tier="haiku")),
                    tasks.parse(write_task(root, "T-0003", status="done", tier=None)),
                    tasks.parse(write_task(root, "T-0004", tier="sonnet")),
                ]
                errors = tasks.validate(parsed)
        self.assertEqual(sum("tier=" in e for e in errors), 2)
        self.assertTrue(any("T-0001" in e for e in errors) and any("T-0002" in e for e in errors))

    def test_next_filters_by_tier_and_takes_handoffs_first(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus")
            write_task(root, "T-0002", tier="opus", body="\n## Handoff\nsonnet run stuck on X\n")
            write_task(root, "T-0003", tier="sonnet")
            self.assertEqual([run_next(root, "opus"), run_next(root, "sonnet")], ["T-0002", "T-0003"])

    def test_earlier_milestone_comes_before_priority_and_m9_still_gets_picked(self) -> None:
        # Decision 0026: an M2 function beats an M9 last-digit tail of higher priority.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0901", tier="opus", priority="P1", milestone="M9")
            write_task(root, "T-0210", tier="opus", priority="P2", milestone="M2")
            write_task(root, "T-0110", tier="opus", priority="P3", milestone="M10")
            self.assertEqual(run_next(root, "opus"), "T-0210")
            claims = ({"T-0210"}, set())
            self.assertEqual(run_next(root, "opus", claims=claims), "T-0901")

    def test_campaign_task_comes_before_milestone_order_but_after_p0_and_handoffs(self) -> None:
        # Decision 0083: a visible-parity M10 task beats an M2 task; P0 and handoffs still lead.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0210", tier="opus", priority="P1", milestone="M2")
            write_task(root, "T-1711", tier="opus", priority="P3", milestone="M10", campaign="visible")
            self.assertEqual(run_next(root, "opus"), "T-1711")
            write_task(root, "T-0002", tier="opus", priority="P2", body="\n## Handoff\nstuck\n")
            self.assertEqual(run_next(root, "opus"), "T-0002")
            write_task(root, "T-0001", tier="opus", priority="P0", milestone="M9")
            self.assertEqual(run_next(root, "opus"), "T-0001")

    def test_open_dependency_of_a_campaign_task_is_picked_as_campaign(self) -> None:
        # Decision 0094: a sweep the campaign fix waits on beats milestone order; a done one does not count.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0210", tier="opus", priority="P1", milestone="M2")
            write_task(root, "T-2145", tier="opus", priority="P3", milestone="M9", lane="research", depends_on="T-2144")
            write_task(root, "T-2144", tier="opus", priority="P3", milestone="M9", lane="research")
            write_task(root, "T-2146", tier="opus", milestone="M9", lane="functions-stat", depends_on="T-2145",
                       campaign="visible")
            self.assertEqual(run_next(root, "opus"), "T-2144")
            write_task(root, "T-2144", tier="opus", priority="P3", milestone="M9", lane="research", status="done")
            self.assertEqual(run_next(root, "opus"), "T-2145")
            write_task(root, "T-2146", tier="opus", milestone="M9", lane="functions-stat", depends_on="T-2145",
                       campaign="visible", status="done")
            self.assertEqual(run_next(root, "opus"), "T-0210")

    def test_campaign_stages_order_semantics_then_accuracy_then_visible(self) -> None:
        # Decision 0114: the stage beats milestone and priority among campaign tasks.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0210", tier="opus", priority="P0", milestone="M2", status="done")
            write_task(root, "T-0501", tier="opus", priority="P1", milestone="M5", campaign="visible")
            write_task(root, "T-0901", tier="opus", priority="P3", milestone="M9", campaign="accuracy")
            write_task(root, "T-0902", tier="opus", priority="P3", milestone="M9", campaign="semantics")
            self.assertEqual(run_next(root, "opus"), "T-0902")
            self.assertEqual(run_next(root, "opus", claims=({"T-0902"}, set())), "T-0901")
            self.assertEqual(run_next(root, "opus", claims=({"T-0902", "T-0901"}, set())), "T-0501")

    def test_dependency_takes_the_earliest_stage_that_needs_it(self) -> None:
        # Decision 0114: a sweep that a semantics task waits on comes before an accuracy task.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0901", tier="opus", milestone="M9", campaign="accuracy")
            write_task(root, "T-0950", tier="opus", milestone="M9", lane="research")
            write_task(root, "T-0951", tier="opus", milestone="M9", lane="functions-stat", depends_on="T-0950",
                       campaign="visible")
            write_task(root, "T-0952", tier="opus", milestone="M9", lane="functions-math", depends_on="T-0950",
                       campaign="semantics")
            self.assertEqual(run_next(root, "opus"), "T-0950")

    def test_deferred_task_is_never_ready_but_validates(self) -> None:
        # Decision 0114: `defer: phase-1b` parks last-digit work; an unknown deferral is rejected.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", priority="P0", defer="phase-1b")
            write_task(root, "T-0002", tier="opus", priority="P3", milestone="M9")
            self.assertEqual(run_next(root, "opus"), "T-0002")
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(root / "docs/tasks/M0/T-0001-x.md"),
                    tasks.parse(write_task(root, "T-0003", defer="later")),
                ]
                errors = tasks.validate(parsed)
        self.assertTrue(any("defer=" in e and "T-0003" in e for e in errors))
        self.assertFalse(any("T-0001" in e for e in errors))

    def test_unknown_campaign_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(write_task(root, "T-0001", campaign="visible")),
                    tasks.parse(write_task(root, "T-0002", campaign="bits")),
                ]
                errors = tasks.validate(parsed)
        self.assertTrue(any("campaign=" in e and "T-0002" in e for e in errors))
        self.assertFalse(any("T-0001" in e for e in errors))

    def test_task_waiting_on_an_open_sweep_is_not_ready(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0901", tier="opus", milestone="M9", depends_on="T-0942")
            write_task(root, "T-0942", tier="opus", milestone="M9", lane="research")
            write_task(root, "T-0939", tier="opus", milestone="M9", priority="P2")
            self.assertEqual(run_next(root, "opus", lane="harness"), "T-0939")
            write_task(root, "T-0942", tier="opus", milestone="M9", lane="research", status="done")
            self.assertEqual(run_next(root, "opus", lane="harness"), "T-0901")

    def test_p0_comes_before_a_handoff(self) -> None:
        # T-0931 (P0 stack overflow) once waited behind the T-0125 (P2) handoff.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", priority="P2", body="\n## Handoff\nsonnet run stuck\n")
            write_task(root, "T-0002", tier="opus", priority="P0")
            write_task(root, "T-0003", tier="opus", priority="P1")
            self.assertEqual(run_next(root, "opus"), "T-0002")

    def test_handoff_seen_only_on_its_branch_moves_the_task_to_opus(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus")
            write_task(root, "T-0002", tier="sonnet")  # main still says sonnet
            write_task(root, "T-0003", tier="sonnet")
            claims = ({"T-0002"}, {"T-0002"})  # pushed branch with a Handoff
            self.assertEqual(run_next(root, "opus", claims), "T-0002")
            self.assertEqual(run_next(root, "sonnet", claims), "T-0003")

    def test_resumed_handoff_is_no_longer_a_handoff(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus")
            write_task(root, "T-0002", tier="opus", body="\n## Handoff (resumed)\nopus run picked it up\n")
            # Resumed: its fresh branch is a claim again, so the next opus run skips it.
            self.assertEqual(run_next(root, "opus", ({"T-0002"}, set())), "T-0001")

    def test_claimed_task_is_skipped(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet")
            write_task(root, "T-0002", tier="sonnet")
            self.assertEqual(run_next(root, "sonnet", ({"T-0001"}, set())), "T-0002")

    def test_review_limited_task_is_resumed_by_opus_after_handoffs_and_before_new_work(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", campaign="visible")
            write_task(root, "T-0002", tier="sonnet", lane="engine")  # blocked on its branch
            write_task(root, "T-0003", tier="opus", body="\n## Handoff\nstuck\n")
            write_task(root, "T-0004", tier="sonnet")
            claims = ({"T-0002"}, set())
            limited = {"T-0002": "agent/engine/T-0002-x"}
            busy = frozenset({("engine", "T-0009")})
            self.assertEqual(run_next(root, "opus", claims, busy, limited=limited), "T-0003")
            (root / "docs/tasks/M0/T-0003-x.md").unlink()
            line = run_next_line(root, "opus", claims, busy, limited=limited)
            self.assertEqual(line.split()[0], "T-0002")
            self.assertIn("resume agent/engine/T-0002-x", line)
            # Sonnet never re-reviews it, and a plain blocked claim is not offered at all.
            self.assertEqual(run_next(root, "sonnet", claims, limited=limited), "T-0004")
            self.assertEqual(run_next(root, "opus", claims), "T-0001")

    def test_ready_lists_what_next_could_pick_and_counts_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet")
            write_task(root, "T-0002", tier="sonnet")  # claimed by a pushed branch
            write_task(root, "T-0003", tier="opus")
            write_task(root, "T-0004", tier="sonnet", status="done")
            with mock.patch.object(tasks, "ROOT", root), \
                    mock.patch.object(tasks, "TASKS", root / "docs/tasks"), \
                    mock.patch.object(tasks.sys, "argv", ["tasks.py", "ready", "--tier", "sonnet"]), \
                    mock.patch.object(tasks, "branch_claims", return_value=({"T-0002"}, set(), set(), {})), \
                    mock.patch("builtins.print") as printed:
                self.assertEqual(tasks.main(), 0)
        lines = [str(c[0][0]) for c in printed.call_args_list]
        self.assertEqual([lines[0].split()[0], lines[-1]], ["T-0001", "1 ready"])


class FunctionLanes(unittest.TestCase):
    def test_busy_lane_is_skipped_and_functions_means_any_sub_lane(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-text")
            write_task(root, "T-0002", tier="sonnet", lane="functions-math")
            write_task(root, "T-0003", tier="sonnet", lane="engine")
            self.assertEqual(run_next(root, "sonnet", lane="functions"), "T-0001")
            busy_text = frozenset({("functions-text", "T-0009")})  # T-0009: open task on another branch
            self.assertEqual(run_next(root, "sonnet", lane="functions", active=busy_text), "T-0002")
            self.assertEqual(run_next(root, "sonnet", lane="functions-math"), "T-0002")

    def test_handoff_is_resumed_even_in_a_busy_lane(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-stat", body="\n## Handoff\nstuck\n")
            self.assertEqual(run_next(root, "opus", active=frozenset({("functions-stat", "T-0008")})), "T-0001")

    def test_branch_of_a_done_task_does_not_keep_its_lane_busy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-text", status="done")
            write_task(root, "T-0002", tier="sonnet", lane="functions-text")
            # T-0001 merged a minute ago and its branch still exists: the lane is free again.
            self.assertEqual(run_next(root, "sonnet", active=frozenset({("functions-text", "T-0001")})), "T-0002")

    def test_busy_lane_comes_from_the_tasks_own_lane_not_the_branch_name(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-text", status="in-progress")
            write_task(root, "T-0002", tier="sonnet", lane="functions-text")
            write_task(root, "T-0003", tier="sonnet", lane="functions-math")
            # A branch pushed as agent/functions/T-0001-... still makes functions-text busy.
            self.assertEqual(run_next(root, "sonnet", active=frozenset({("functions", "T-0001")})), "T-0003")

    def test_plain_functions_lane_is_retired_for_open_tasks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(write_task(root, "T-0001", lane="functions")),
                    tasks.parse(write_task(root, "T-0002", status="done", lane="functions")),
                    tasks.parse(write_task(root, "T-0003", lane="functions-lookup")),
                ]
                errors = tasks.validate(parsed)
        self.assertEqual([e for e in errors if "retired" in e and "T-0001" in e], errors)
        self.assertEqual(len(errors), 1)


T = "TODO"


class DependencyCycles(unittest.TestCase):
    """`validate` reports each `depends_on` cycle once (T-0091), whatever the tasks' status."""

    @staticmethod
    def cycles(graph: dict[str, list[str]], done: tuple[str, ...] = ()) -> list[str]:
        parsed = [
            {"id": tid, "depends_on": deps, "status": "done" if tid in done else "todo"}
            for tid, deps in graph.items()
        ]
        return [" -> ".join(c + c[:1]) for c in tasks.dependency_cycles(parsed)]

    def test_two_and_three_task_cycles_are_reported_once_from_the_smallest_id(self) -> None:
        self.assertEqual(self.cycles({"T-0965": ["T-0963"], "T-0963": ["T-0965"]}), ["T-0963 -> T-0965 -> T-0963"])
        three = {"T-0003": ["T-0001"], "T-0001": ["T-0002"], "T-0002": ["T-0003"], "T-0004": ["T-0002"]}
        self.assertEqual(self.cycles(three, done=("T-0002",)), ["T-0001 -> T-0002 -> T-0003 -> T-0001"])

    def test_a_self_dependency_is_a_cycle(self) -> None:
        self.assertEqual(self.cycles({"T-0001": ["T-0001"], "T-0002": ["T-0001"]}), ["T-0001 -> T-0001"])

    def test_an_acyclic_diamond_and_an_unknown_dependency_are_no_cycle(self) -> None:
        diamond = {"T-0001": ["T-0002", "T-0003"], "T-0002": ["T-0004"], "T-0003": ["T-0004"], "T-0004": ["T-9999"]}
        self.assertEqual(self.cycles(diamond), [])

    def test_validate_reports_the_cycle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(write_task(root, "T-0001", depends_on="T-0002")),
                    tasks.parse(write_task(root, "T-0002", depends_on="T-0001")),
                ]
                errors = tasks.validate(parsed)
        self.assertEqual(errors, ["T-0001 -> T-0002 -> T-0001: depends_on cycle"])


class StaleTodos(unittest.TestCase):
    # The TODOs are built from T so this file holds none for the check itself to flag.
    TASKS = [
        {"id": "T-0001", "status": "todo"},
        {"id": "T-0002", "status": "in-progress"},
        {"id": "T-0003", "status": "blocked"},
        {"id": "T-0004", "status": "done"},
        {"id": "T-0005", "status": "dropped"},
    ]

    def test_todos_naming_open_tasks_pass(self) -> None:
        files = {"a.rs": f"// {T}(T-0001): x\n// {T}(T-0002) y\n", "b.py": f"# {T}(T-0003)\n"}
        self.assertEqual(tasks.stale_todos(self.TASKS, files), [])

    def test_todos_naming_closed_or_unknown_tasks_fail_with_file_and_line(self) -> None:
        files = {"src/a.rs": f"fn f() {{}}\n// {T}(T-0004): x\n", "b.py": f"x = 1  # {T}(T-0005) and {T}(T-9999)\n"}
        errors = tasks.stale_todos(self.TASKS, files)
        self.assertEqual(len(errors), 3)
        self.assertTrue(errors[0].startswith(f"b.py:1: {T}(T-0005)") and "is dropped" in errors[0])
        self.assertTrue(errors[1].startswith(f"b.py:1: {T}(T-9999)") and "does not exist" in errors[1])
        self.assertTrue(errors[2].startswith(f"src/a.rs:2: {T}(T-0004)") and "is done" in errors[2])

    def test_bare_todos_and_template_placeholders_are_not_matched(self) -> None:
        files = {"a.rs": f"// TODO: x\n// {T}(fill)\n// {T}(T-XXXX)\n// {T}(T-0123x)\n"}
        self.assertEqual(tasks.stale_todos(self.TASKS, files), [])


class BranchClaims(unittest.TestCase):
    def test_recent_branches_claim_old_ones_only_count_with_a_handoff(self) -> None:
        now = int(time.time())
        refs = (
            f"refs/remotes/origin/agent/engine/T-0001-new {now - 60}\n"
            f"refs/remotes/origin/agent/engine/T-0002-stale {now - 9 * 86400}\n"
            f"refs/remotes/origin/agent/harness/T-0003-handed {now - 9 * 86400}\n"
        )

        def fake_run(cmd: list[str], **_: object) -> mock.Mock:
            args = cmd[3:]
            if args[0] == "for-each-ref":
                return mock.Mock(stdout=refs)
            if args[0] == "ls-tree":
                tid = args[3].rsplit("/", 1)[-1][:6]
                return mock.Mock(stdout=f"docs/tasks/M0/{tid}-x.md\n")
            body = "\n## Handoff\nstuck\n" if "T-0003" in args[1] else "plain\n"
            return mock.Mock(stdout=body)

        with mock.patch.object(tasks.subprocess, "run", side_effect=fake_run):
            # T-0001 is recent, so its lane is busy; T-0003's handoff waits for opus and keeps
            # harness free.
            self.assertEqual(
                tasks.branch_claims(fetch=False), ({"T-0001"}, {"T-0003"}, {("engine", "T-0001")}, {})
            )

    def test_blocked_task_on_its_branch_frees_the_lane_but_stays_claimed(self) -> None:
        now = int(time.time())
        refs = (
            f"refs/remotes/origin/agent/engine/T-0001-working {now - 60}\n"
            f"refs/remotes/origin/agent/research/T-0002-blocked {now - 60}\n"
        )

        def fake_run(cmd: list[str], **_: object) -> mock.Mock:
            args = cmd[3:]
            if args[0] == "for-each-ref":
                return mock.Mock(stdout=refs)
            if args[0] == "ls-tree":
                tid = args[3].rsplit("/", 1)[-1][:6]
                return mock.Mock(stdout=f"docs/tasks/M0/{tid}-x.md\n")
            status = "blocked" if "T-0002" in args[1] else "in-progress"
            return mock.Mock(stdout=f"---\nid: x\nstatus: {status}\n---\nbody\n")

        with mock.patch.object(tasks.subprocess, "run", side_effect=fake_run):
            # T-0002 was blocked a minute ago: nobody restarts it, but research may take its next task.
            self.assertEqual(
                tasks.branch_claims(fetch=False), ({"T-0001", "T-0002"}, set(), {("engine", "T-0001")}, {})
            )

    def test_task_blocked_on_a_stale_branch_stays_claimed(self) -> None:
        stale = int(time.time()) - 3 * tasks.CLAIM_TTL_SECONDS
        refs = (
            f"refs/remotes/origin/agent/research/T-0002-blocked {stale}\n"
            f"refs/remotes/origin/agent/engine/T-0001-abandoned {stale}\n"
        )

        def fake_run(cmd: list[str], **_: object) -> mock.Mock:
            args = cmd[3:]
            if args[0] == "for-each-ref":
                return mock.Mock(stdout=refs)
            if args[0] == "ls-tree":
                tid = args[3].rsplit("/", 1)[-1][:6]
                return mock.Mock(stdout=f"docs/tasks/M0/{tid}-x.md\n")
            status = "blocked" if "T-0002" in args[1] else "in-progress"
            return mock.Mock(stdout=f"---\nid: x\nstatus: {status}\n---\nbody\n")

        with mock.patch.object(tasks.subprocess, "run", side_effect=fake_run):
            # A blocked task stays claimed however old its branch is (decision 0048); an abandoned
            # in-progress branch past the TTL frees its task.
            self.assertEqual(tasks.branch_claims(fetch=False), ({"T-0002"}, set(), set(), {}))

    def test_review_limited_branch_is_offered_once_idle_and_only_on_the_first_block(self) -> None:
        now = int(time.time())
        idle = now - tasks.BUSY_SECONDS - 60
        refs = (
            f"refs/remotes/origin/agent/engine/T-0001-limit {idle}\n"
            f"refs/remotes/origin/agent/engine/T-0002-limit-recent {now - 60}\n"
            f"refs/remotes/origin/agent/harness/T-0003-twice {idle}\n"
            f"refs/remotes/origin/agent/harness/T-0004-other-block {idle}\n"
            f"refs/remotes/origin/agent/harness/T-0005-$(touch-x) {idle}\n"
        )
        reasons = {
            "T-0001": "review-round limit",
            "T-0002": "review-round limit",
            "T-0003": "review-round limit, twice",
            "T-0004": "time-boxed (decision 0056)",
            "T-0005": "review-round limit",
        }

        def fake_run(cmd: list[str], **_: object) -> mock.Mock:
            args = cmd[3:]
            if args[0] == "for-each-ref":
                return mock.Mock(stdout=refs)
            if args[0] == "ls-tree":
                tid = args[3].rsplit("/", 1)[-1][:6]
                return mock.Mock(stdout=f"docs/tasks/M0/{tid}-x.md\n")
            tid = next(t for t in reasons if t in args[1].split(":")[0])
            body = f"---\nid: x\nstatus: blocked\n---\n## Blocked because\nBlocked because: {reasons[tid]}\n"
            return mock.Mock(stdout=body)

        with mock.patch.object(tasks.subprocess, "run", side_effect=fake_run):
            claimed, _, _, limited = tasks.branch_claims(fetch=False)
        self.assertEqual(claimed, set(reasons))
        # T-0002 moved a minute ago (its run may still be at it); T-0003 is the owner's now; T-0005's
        # branch name is not one `next` will print for an agent to check out.
        self.assertEqual(limited, {"T-0001": "agent/engine/T-0001-limit"})


class Stats(unittest.TestCase):
    def run_stats(self, root: Path, *argv: str, files: dict[str, str] | None = None) -> tuple[int, str, str]:
        """`tasks.py stats` on `root`'s tasks and the code `files` (path -> text, standing in for
        `code_files`): (exit code, stdout, stderr). Any other git call fails the test."""
        out, err = io.StringIO(), io.StringIO()
        no_git = AssertionError("stats must not fetch or read branches")
        with mock.patch.object(tasks, "ROOT", root), mock.patch.object(tasks, "TASKS", root / "docs/tasks"), \
                mock.patch.object(tasks.sys, "argv", ["tasks.py", "stats", *argv]), \
                mock.patch.object(tasks, "code_files", return_value=files or {}), \
                mock.patch.object(tasks, "branch_claims", side_effect=no_git), \
                mock.patch.object(tasks, "git_branch_names", side_effect=no_git), \
                mock.patch.object(tasks.subprocess, "run", side_effect=no_git), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = tasks.main()
        return code, out.getvalue(), err.getvalue()

    def test_counts_per_milestone_campaign_stage_and_deferral(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", status="done")
            write_task(root, "T-0002", depends_on="T-0001", campaign="semantics")  # ready
            write_task(root, "T-0003", depends_on="T-0004", campaign="semantics")  # waits on T-0004
            write_task(root, "T-0004", status="blocked", campaign="semantics")
            write_task(root, "T-0005", status="dropped", campaign="visible")
            write_task(root, "T-0006", status="done", campaign="visible", milestone="M10")
            write_task(root, "T-0007", campaign="accuracy", defer="phase-1b", milestone="M10")  # never ready
            write_task(root, "T-0008", status="done", defer="phase-1b", milestone="M10")
            write_task(root, "T-0009", status="dropped", defer="phase-1b", milestone="M2")
            write_task(root, "T-0010", status="in-progress", campaign="accuracy", milestone="M2")
            code, out, err = self.run_stats(root, "--json")
            rerun = self.run_stats(root, "--json")[1]
        self.assertEqual((code, err), (0, ""))
        zero = dict.fromkeys(tasks.STATUSES, 0)
        self.assertEqual(json.loads(out), {
            "by_milestone": {
                "M0": {**zero, "todo": 2, "blocked": 1, "done": 1, "dropped": 1},
                "M10": {**zero, "todo": 1, "done": 2},
                "M2": {**zero, "in-progress": 1, "dropped": 1},
            },
            "by_campaign": {
                "semantics": {**zero, "todo": 2, "blocked": 1, "ready": 1},
                "accuracy": {**zero, "todo": 1, "in-progress": 1, "ready": 0},
                "visible": {**zero, "done": 1, "dropped": 1, "ready": 0},
            },
            "deferred": {**zero, "todo": 1, "done": 1, "dropped": 1, "ready": 0},
        })
        # Deterministic: keys sorted at every level, and the same bytes on a rerun.
        self.assertEqual(rerun, out)
        self.assertLess(out.index('"by_campaign"'), out.index('"by_milestone"'))
        self.assertLess(out.index('"M10"'), out.index('"M2"'))

    def test_ready_ignores_branch_claims(self) -> None:
        # `next` would skip a claimed task; the dashboard's count does not look at branches.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", campaign="visible")
            _, out, _ = self.run_stats(root, "--json")
        self.assertEqual(json.loads(out)["by_campaign"]["visible"]["ready"], 1)

    def test_fails_on_what_validate_reports_and_prints_no_counts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", campaign="bits")
            write_task(root, "T-0002", depends_on="T-0999")
            code, out, err = self.run_stats(root, "--json")
        self.assertEqual((code, out), (1, ""))
        self.assertIn("campaign='bits'", err)
        self.assertIn("unknown dependency T-0999", err)

    def test_fails_on_a_stale_todo_like_validate(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", status="done")
            code, out, err = self.run_stats(root, "--json", files={"a.rs": f"// {T}(T-0001): x\n"})
        self.assertEqual((code, out), (1, ""))
        self.assertIn("names a task that is done", err)

    def test_only_git_call_is_a_local_ls_files(self) -> None:
        # The real `code_files` runs; record every subprocess call it makes.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001")
            (root / "scripts").mkdir()
            (root / "scripts/a.py").write_text(f"# {T}(T-0001): x\n")
            calls: list[list[str]] = []

            def record(cmd: list[str], **_: object) -> mock.Mock:
                calls.append(cmd)
                return mock.Mock(stdout="scripts/a.py\0")

            argv = ["tasks.py", "stats", "--json"]
            with mock.patch.object(tasks, "ROOT", root), mock.patch.object(tasks, "TASKS", root / "docs/tasks"), \
                    mock.patch.object(tasks.sys, "argv", argv), \
                    mock.patch.object(tasks.subprocess, "run", side_effect=record), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(tasks.main(), 0)
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][:4], ["git", "-C", str(root), "ls-files"])

    def test_missing_git_fails_cleanly(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001")
            err = io.StringIO()
            argv = ["tasks.py", "stats", "--json"]
            with mock.patch.object(tasks, "ROOT", root), mock.patch.object(tasks, "TASKS", root / "docs/tasks"), \
                    mock.patch.object(tasks.sys, "argv", argv), \
                    mock.patch.object(tasks.subprocess, "run", side_effect=FileNotFoundError("git")), \
                    contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(err):
                self.assertEqual(tasks.main(), 1)
        self.assertEqual(out.getvalue(), "")
        self.assertIn("git ls-files", err.getvalue())

    def test_needs_json(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            code, out, err = self.run_stats(Path(tmp))
        self.assertEqual((code, out), (2, ""))
        self.assertIn("usage", err)


class Slug(unittest.TestCase):
    def test_slug(self) -> None:
        self.assertEqual(tasks.slug("Parser: bound `nesting` depth!"), "parser-bound-nesting-depth")


if __name__ == "__main__":
    unittest.main()
