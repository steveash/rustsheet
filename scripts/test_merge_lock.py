"""Unit tests for scripts/merge_lock.py against a real temporary remote (stdlib only; run by
`make hygiene`)."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_lock  # noqa: E402 (the sys.path line above must run first)


def run(*args: str, cwd: Path) -> str:
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout


class MergeLock(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        base = Path(self.tmp.name)
        self.origin = base / "origin.git"
        run("git", "init", "-q", "--bare", str(self.origin), cwd=base)
        # Two agents' clones of the same origin.
        self.a, self.b = base / "a", base / "b"
        for clone in (self.a, self.b):
            run("git", "clone", "-q", str(self.origin), str(clone), cwd=base)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def lock(self) -> merge_lock.Lock | None:
        return merge_lock.read_lock(self.a)

    def test_acquire_takes_a_free_lock_with_an_empty_tree(self) -> None:
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 0)
        lock = self.lock()
        assert lock is not None
        self.assertEqual((lock.holder, lock.pr), ("agent-a", "101"))
        self.assertEqual(run("git", "ls-tree", lock.sha, cwd=self.a), "")

    def test_a_live_lock_is_busy_for_another_holder(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 1)
        lock = self.lock()
        assert lock is not None
        self.assertEqual(lock.holder, "agent-a")

    def test_of_two_racing_acquirers_exactly_one_wins(self) -> None:
        # Both read a free lock, then both push: the server's lease check refuses the second.
        with mock.patch.object(merge_lock, "read_lock", side_effect=[None, None]):
            first = merge_lock.acquire("agent-a", "101", root=self.a)
        with mock.patch.object(merge_lock, "read_lock", side_effect=[None, self.lock()]):
            second = merge_lock.acquire("agent-b", "202", root=self.b)
        self.assertEqual((first, second), (0, 1))
        lock = self.lock()
        assert lock is not None
        self.assertEqual(lock.holder, "agent-a")

    def test_the_holder_renews_its_own_lock(self) -> None:
        with mock.patch.dict(os.environ, {"GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"}):
            merge_lock.acquire("agent-a", "101", root=self.a)
        before = self.lock()
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 0)
        after = self.lock()
        assert before is not None and after is not None
        self.assertGreater(after.taken_at, before.taken_at)
        self.assertEqual(after.holder, "agent-a")

    def test_a_lock_past_its_ttl_is_taken_over(self) -> None:
        with mock.patch.dict(os.environ, {"GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"}):
            merge_lock.acquire("gone", "1", root=self.a)
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 0)
        lock = self.lock()
        assert lock is not None
        self.assertEqual((lock.holder, lock.pr), ("agent-b", "202"))

    def test_release_drops_only_the_callers_lock(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        self.assertEqual(merge_lock.release("agent-b", root=self.b), 1)
        self.assertIsNotNone(self.lock())
        self.assertEqual(merge_lock.release("agent-a", root=self.a), 0)
        self.assertIsNone(self.lock())
        self.assertEqual(merge_lock.release("agent-a", root=self.a), 0)

    def test_a_lock_taken_over_before_release_is_left_alone(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        stale = self.lock()
        merge_lock.acquire("agent-b", "202", ttl_minutes=0, root=self.b)
        with mock.patch.object(merge_lock, "read_lock", return_value=stale):
            self.assertEqual(merge_lock.release("agent-a", root=self.a), 0)
        lock = self.lock()
        assert lock is not None
        self.assertEqual(lock.holder, "agent-b")


if __name__ == "__main__":
    unittest.main()
