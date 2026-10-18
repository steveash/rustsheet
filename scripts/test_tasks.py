"""Unit tests for scripts/tasks.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

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
) -> Path:
    path = root / "docs/tasks" / milestone / f"{tid}-x.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    tier_line = f"tier: {tier}\n" if tier else ""
    path.write_text(
        f"---\nid: {tid}\ntitle: \"x\"\nmilestone: {milestone}\nlane: {lane}\nstatus: {status}\npriority: {priority}\n"
        f"size: S\n{tier_line}depends_on: [{depends_on}]\n---\n{body}"
    )
    return path


def run_next(
    root: Path,
    tier: str,
    claims: tuple[set[str], set[str]] = (set(), set()),
    active: frozenset[tuple[str, str]] = frozenset(),
    lane: str | None = None,
) -> str:
    argv = ["tasks.py", "next", "--tier", tier, *(["--lane", lane] if lane else [])]
    with mock.patch.object(tasks, "ROOT", root), mock.patch.object(tasks, "TASKS", root / "docs/tasks"), \
            mock.patch.object(tasks.sys, "argv", argv), \
            mock.patch.object(tasks, "branch_claims", return_value=(*claims, set(active))), \
            mock.patch("builtins.print") as printed:
        tasks.main()
    return str(printed.call_args[0][0]).split()[0]


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
                    mock.patch.object(tasks, "branch_claims", return_value=({"T-0002"}, set(), set())), \
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
            self.assertEqual(tasks.branch_claims(fetch=False), ({"T-0001"}, {"T-0003"}, {("engine", "T-0001")}))

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
                tasks.branch_claims(fetch=False), ({"T-0001", "T-0002"}, set(), {("engine", "T-0001")})
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
            self.assertEqual(tasks.branch_claims(fetch=False), ({"T-0002"}, set(), set()))


class Slug(unittest.TestCase):
    def test_slug(self) -> None:
        self.assertEqual(tasks.slug("Parser: bound `nesting` depth!"), "parser-bound-nesting-depth")


if __name__ == "__main__":
    unittest.main()
