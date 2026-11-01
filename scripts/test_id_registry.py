"""Tests for id_registry.py (decision 0215) against a real temporary origin (stdlib only)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_decisions  # noqa: E402 (the sys.path line above must run first)
import id_registry  # noqa: E402
import merge_lock  # noqa: E402
import new_decision  # noqa: E402
import tasks  # noqa: E402

REAL_GIT = merge_lock.git


def run(*args: str, cwd: Path) -> str:
    # No stdin: `git mktree` (seed) reads it, so an inherited open pipe would hang the test (T-3694).
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout


def next_after(base: int) -> object:
    """A `pick` rule: one above the highest of `base` and the claims, as `T-NNNN`."""
    return lambda claimed: f"T-{max([base, *(int(c[2:]) for c in claimed if c.startswith('T-'))]) + 1:04d}"


class IdRegistry(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        self.origin = base / "origin.git"
        run("git", "init", "-q", "--bare", str(self.origin), cwd=base)
        self.a, self.b = base / "a", base / "b"
        for clone in (self.a, self.b):
            run("git", "clone", "-q", str(self.origin), str(clone), cwd=base)

        def guarded(*args: str, root: Path = merge_lock.ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
            if Path(root).resolve() == merge_lock.ROOT:
                raise AssertionError(f"test ran git {args[:1]} in the real repository")
            return REAL_GIT(*args, root=root, check=check)

        for module in (merge_lock, id_registry):
            patcher = mock.patch.object(module, "git", side_effect=guarded)
            patcher.start()
            self.addCleanup(patcher.stop)

    def test_claims_are_recorded_in_order_from_an_absent_registry(self) -> None:
        self.assertEqual(id_registry.read(self.a), ("", []))
        self.assertEqual(id_registry.claim(next_after(3500), id_registry.TASK, self.a), "T-3501")
        self.assertEqual(id_registry.claim(next_after(3500), id_registry.TASK, self.b), "T-3502")  # b sees a's claim
        # Each kind sees only its own claims: a decision rule never gets a task id, and back.
        self.assertEqual(id_registry.claim(lambda claimed: f"{len(claimed) + 213:04d}", id_registry.DECISION, self.a), "0213")
        self.assertEqual(id_registry.claim(next_after(3500), id_registry.TASK, self.b), "T-3503")
        self.assertEqual(id_registry.read(self.b)[1], ["T-3501", "T-3502", "0213", "T-3503"])

    def test_a_session_that_loses_the_race_takes_the_next_id(self) -> None:
        real_push = merge_lock.push_lease
        raced = False

        def push(expected: str, new: str, root: Path, ref: str = merge_lock.LOCK_REF) -> bool:
            nonlocal raced
            if not raced:  # b claims between a's read and a's push
                raced = True
                self.assertEqual(id_registry.claim(next_after(3500), id_registry.TASK, self.b), "T-3501")
            return real_push(expected, new, root, ref=ref)

        with mock.patch.object(merge_lock, "push_lease", push):
            self.assertEqual(id_registry.claim(next_after(3500), id_registry.TASK, self.a), "T-3502")
        self.assertEqual(id_registry.read(self.a)[1], ["T-3501", "T-3502"])

    def test_a_pick_of_a_claimed_or_malformed_id_is_refused(self) -> None:
        id_registry.claim(lambda claimed: "T-3501", id_registry.TASK, self.a)
        for bad in ("T-3501", "T-35", "../x", "0214"):  # 0214: a decision number from a task rule
            with self.subTest(bad=bad), self.assertRaises(id_registry.RegistryError):
                id_registry.claim(lambda claimed, bad=bad: bad, id_registry.TASK, self.a)

    def test_only_the_newest_claims_are_kept(self) -> None:
        with mock.patch.object(id_registry, "KEEP", 3):
            for _ in range(5):
                id_registry.claim(next_after(3500), id_registry.TASK, self.a)
        self.assertEqual(id_registry.read(self.b)[1], ["T-3503", "T-3504", "T-3505"])

    def test_allocate_falls_back_to_the_scan_offline_or_when_the_registry_fails(self) -> None:
        with mock.patch("sys.stderr"):
            self.assertEqual(id_registry.allocate(next_after(3500), id_registry.TASK, fetch=False, root=self.a), ("T-3501", False))
            self.assertEqual(id_registry.read(self.a)[1], [])  # offline: nothing reserved
            run("git", "remote", "set-url", "origin", str(Path(self.tmp.name) / "missing.git"), cwd=self.a)
            self.assertEqual(id_registry.allocate(next_after(3500), id_registry.TASK, fetch=True, root=self.a), ("T-3501", False))
        run("git", "remote", "set-url", "origin", str(self.origin), cwd=self.a)
        self.assertEqual(id_registry.allocate(next_after(3500), id_registry.TASK, fetch=True, root=self.a), ("T-3501", True))
        self.assertEqual(id_registry.read(self.b)[1], ["T-3501"])


    def seed(self, *lines: str) -> None:
        """Pushes a registry commit holding `lines` (claims, junk, anything) from clone b."""
        tree = run("git", "mktree", cwd=self.b).strip()
        message = "\n".join([id_registry.HEADER, "", *lines])
        sha = run("git", "-c", "user.name=t", "-c", "user.email=t@invalid", "commit-tree", tree, "-m", message,
                  cwd=self.b).strip()
        run("git", "push", "-q", "-f", "origin", f"{sha}:{id_registry.REF}", cwd=self.b)

    def test_both_allocators_reserve_through_the_registry_past_junk_and_the_other_kind(self) -> None:
        """`make new-task` and `make new-decision` end to end against an origin (the int("T-3543")
        crash of b1bc750 was in this path)."""
        self.seed("T-0005", "0007", "not a claim", "T-0105", "../../etc", "0003")
        decisions = self.a / "docs" / "decisions"
        decisions.mkdir(parents=True)
        (decisions / "0001-a.md").write_text("x\n")
        argv = ["tasks.py", "new", "--milestone", "M0", "--lane", "harness", "--title", "Do it"]
        with mock.patch.object(tasks, "ROOT", self.a), mock.patch.object(tasks, "TASKS", self.a / "docs/tasks"), \
                mock.patch.object(id_registry, "ROOT", self.a), mock.patch.object(check_decisions, "DECISIONS", decisions), \
                mock.patch.object(tasks.sys, "argv", argv), mock.patch("builtins.print"):
            self.assertEqual(tasks.new_task([{"id": "T-0002", "milestone": "M0"}], fetch=True), 0)
            self.assertEqual(new_decision.new_decision("A rule", fetch=True), 0)
        self.assertTrue((self.a / "docs/tasks/M0/T-0006-do-it.md").exists())  # block 00's mark is T-0005
        self.assertEqual(sorted(p.name[:4] for p in decisions.glob("*.md")), ["0001", "0008"])
        self.assertEqual(id_registry.read(self.b)[1], ["T-0005", "0007", "T-0105", "0003", "T-0006", "0008"])

    def test_pruning_keeps_each_blocks_highest_claim(self) -> None:
        with mock.patch.object(id_registry, "KEEP", 2):
            self.seed("T-0042", "0099", "T-0101")
            id_registry.claim(next_after(3500), id_registry.TASK, self.a)
            id_registry.claim(next_after(3500), id_registry.TASK, self.a)
        # T-0042 (block 00), 0099 and T-0101 (block 01) are pruned from the newest 2 but stay as marks.
        self.assertEqual(id_registry.read(self.b)[1], ["0099", "T-0042", "T-0101", "T-3501", "T-3502"])
        block = id_registry.claim(lambda claimed: f"T-{max(int(c[2:]) for c in claimed if c < 'T-01') + 1:04d}",
                                  id_registry.TASK, self.a)
        self.assertEqual(block, "T-0043")

    def test_two_sessions_creating_the_registry_both_get_an_id(self) -> None:
        real_push = merge_lock.push_lease
        raced = False

        def push(expected: str, new: str, root: Path, ref: str = merge_lock.LOCK_REF) -> bool:
            nonlocal raced
            if not raced and expected == "":
                raced = True
                self.assertEqual(id_registry.claim(next_after(3500), id_registry.TASK, self.b), "T-3501")
            return real_push(expected, new, root, ref=ref)

        with mock.patch.object(merge_lock, "push_lease", push):
            self.assertEqual(id_registry.claim(next_after(3500), id_registry.TASK, self.a), "T-3502")

    def test_tasks_py_alone_falls_back_unreserved(self) -> None:
        real_import = __import__

        def no_registry(name: str, *args: object, **kwargs: object) -> object:
            if name == "id_registry":
                raise ImportError(name)
            return real_import(name, *args, **kwargs)  # type: ignore[arg-type]

        with mock.patch("builtins.__import__", no_registry), mock.patch("sys.stderr"):
            self.assertEqual(tasks.reserve_id(lambda claimed: "T-0009", fetch=True),
                             ("T-0009", "NOT reserved: commit and push it now to claim it"))


if __name__ == "__main__":
    unittest.main()
