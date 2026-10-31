"""Tests for id_registry.py (decision 0215) against a real temporary origin (stdlib only)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import id_registry  # noqa: E402 (the sys.path line above must run first)
import merge_lock  # noqa: E402

REAL_GIT = merge_lock.git


def run(*args: str, cwd: Path) -> str:
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout


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

    def test_only_the_newest_claims_are_kept_and_junk_lines_are_ignored(self) -> None:
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


if __name__ == "__main__":
    unittest.main()
