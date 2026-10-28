"""Unit tests for scripts/merge_lock.py against a real temporary remote (stdlib only; run by
`make hygiene`)."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_lock  # noqa: E402 (the sys.path line above must run first)

REAL_GIT = merge_lock.git


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
        # No test reads the GitHub API; a test that needs a finished PR patches `finished`.
        self.finished: set[str] = set()
        # A fake clock an hour behind, so every timestamp written stays before the committer time.
        self.now = int(time.time()) - 3600
        for patcher in (
            mock.patch.object(merge_lock, "pr_finished", lambda pr, root=None: pr in self.finished),
            mock.patch.object(merge_lock.time, "time", lambda: self.now),
            mock.patch.object(merge_lock, "git", side_effect=self.guarded_git),
        ):
            patcher.start()
            self.addCleanup(patcher.stop)

    @staticmethod
    def guarded_git(
        *args: str, root: Path = merge_lock.ROOT, check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        """Git in the temporary clones only: a test that reaches this repository's real
        `merge-lock` branch on origin fails instead."""
        if Path(root).resolve() == merge_lock.ROOT:
            raise AssertionError(f"test ran git {args[:1]} in the real repository")
        return REAL_GIT(*args, root=root, check=check)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def lock(self) -> merge_lock.Lock | None:
        return merge_lock.read_lock(self.a)

    def held_lock(self) -> merge_lock.Lock:
        lock = self.lock()
        assert lock is not None
        return lock

    def queue(self) -> list[str]:
        lock = self.lock()
        return [w.id for w in lock.waiters] if lock else []

    def tick(self, minutes: float) -> None:
        self.now += int(minutes * 60)

    def push(self, message: str, date: str | None = None) -> None:
        """Puts a hand-written lock commit on origin, as a buggy or hostile writer could."""
        env = {"GIT_COMMITTER_DATE": date} if date else {}
        with mock.patch.dict(os.environ, env):
            tree = merge_lock.git("mktree", root=self.a).stdout.strip()
            sha = merge_lock.git(*merge_lock.IDENTITY, "commit-tree", tree, "-m", message, root=self.a).stdout
        run("git", "push", "-q", "-f", "origin", f"{sha.strip()}:{merge_lock.LOCK_REF}", cwd=self.a)

    def put(self, lock: merge_lock.Lock) -> None:
        self.push(merge_lock.render(lock))

    def first_read(self, view: merge_lock.Lock) -> mock._patch[mock.MagicMock]:
        """The next read returns `view` (what the caller saw before another writer moved the
        lock); later reads are real."""
        real, views = merge_lock.read_lock, iter([view])
        return mock.patch.object(
            merge_lock, "read_lock", side_effect=lambda root: next(views, None) or real(root)
        )

    def test_acquire_takes_a_free_lock_with_an_empty_tree(self) -> None:
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 0)
        lock = self.held_lock()
        self.assertEqual((lock.holder, lock.pr), ("agent-a", "101"))
        self.assertEqual(run("git", "ls-tree", lock.sha, cwd=self.a), "")

    def test_a_live_lock_is_busy_for_another_holder(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 1)
        lock = self.held_lock()
        self.assertEqual(lock.holder, "agent-a")

    def test_of_two_racing_acquirers_exactly_one_wins(self) -> None:
        # Both read a free lock, then both push: the server's lease check refuses the second.
        with mock.patch.object(merge_lock, "read_lock", side_effect=[None, None]):
            first = merge_lock.acquire("agent-a", "101", root=self.a)
        with mock.patch.object(merge_lock, "read_lock", side_effect=[None, self.lock()]):
            second = merge_lock.acquire("agent-b", "202", root=self.b)
        self.assertEqual((first, second), (0, 1))
        lock = self.held_lock()
        self.assertEqual(lock.holder, "agent-a")

    def test_a_finished_pr_is_refused_and_leaves_the_lock_alone(self) -> None:
        self.finished.add("101")
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 3)
        self.assertIsNone(self.lock())

    def test_the_holder_cannot_renew_after_its_pr_merged(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        self.finished.add("101")
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 3)

    def test_a_live_lock_of_a_finished_pr_is_taken_over(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        self.finished.add("101")
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 0)
        lock = self.held_lock()
        self.assertEqual((lock.holder, lock.pr), ("agent-b", "202"))

    def test_the_holder_renews_its_own_lock(self) -> None:
        with mock.patch.dict(os.environ, {"GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"}):
            merge_lock.acquire("agent-a", "101", root=self.a)
        before = self.held_lock()
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 0)
        after = self.held_lock()
        self.assertGreater(after.renewed, before.renewed)
        self.assertEqual(after.holder, "agent-a")

    def test_a_lock_past_its_ttl_is_taken_over(self) -> None:
        with mock.patch.dict(os.environ, {"GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"}):
            merge_lock.acquire("gone", "1", root=self.a)
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 0)
        lock = self.held_lock()
        self.assertEqual((lock.holder, lock.pr), ("agent-b", "202"))

    def test_release_frees_only_the_callers_lock(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        self.assertEqual(merge_lock.release("agent-b", root=self.b), 1)
        self.assertIsNotNone(merge_lock.held(self.lock()))
        self.assertEqual(merge_lock.release("agent-a", root=self.a), 0)
        # The branch stays, holding the free marker: the session git proxy refuses deletion (T-1938).
        lock = self.held_lock()
        self.assertTrue(lock.free)
        self.assertIsNone(merge_lock.held(lock))
        self.assertEqual(merge_lock.release("agent-a", root=self.a), 0)

    def test_a_released_lock_is_acquired_again(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        merge_lock.release("agent-a", root=self.a)
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 0)
        lock = merge_lock.held(self.lock())
        assert lock is not None
        self.assertEqual((lock.holder, lock.pr), ("agent-b", "202"))

    def test_no_push_deletes_the_branch(self) -> None:
        real = merge_lock.git
        pushes: list[tuple[str, ...]] = []

        def spy(*args: str, root: Path = merge_lock.ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
            if args[:1] == ("push",):
                pushes.append(args)
            return real(*args, root=root, check=check)

        with mock.patch.object(merge_lock, "git", side_effect=spy):
            merge_lock.acquire("agent-a", "101", root=self.a)
            merge_lock.release("agent-a", root=self.a)
        self.assertEqual(len(pushes), 2)
        self.assertFalse([p for p in pushes if p[-1].startswith(":")], pushes)

    def test_a_lock_taken_over_before_release_is_left_alone(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        stale = self.lock()
        merge_lock.acquire("agent-b", "202", ttl_minutes=0, root=self.b)
        # The first read is stale, so the write loses; the retry finds the lock taken over.
        with self.first_read(stale):
            self.assertEqual(merge_lock.release("agent-a", root=self.a), 0)
        lock = self.held_lock()
        self.assertEqual(lock.holder, "agent-b")

    def test_status_names_the_holder(self) -> None:
        with mock.patch("builtins.print") as printed:
            merge_lock.status(root=self.a)
            merge_lock.acquire("agent-a", "101", root=self.a)
            merge_lock.status(root=self.a)
            merge_lock.acquire("agent-b", "202", root=self.b)
            merge_lock.status(root=self.a)
        lines = [call.args[0] for call in printed.call_args_list]
        self.assertEqual(lines[0], "merge lock: free")
        self.assertTrue(lines[2].startswith("merge lock held by agent-a for PR 101"), lines[2])
        self.assertEqual(lines[-1], "  1. agent-b for PR 202, waiting 0 min")

    def test_a_lock_commit_without_fields_reads_as_an_unknown_holder(self) -> None:
        self.push("x")
        lock = self.held_lock()
        self.assertEqual(lock.holder, "unknown")

    def test_waiters_are_served_in_order_and_a_releaser_rejoins_at_the_tail(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        self.assertEqual(
            [merge_lock.acquire(h, pr, root=self.b) for h, pr in (("b", "2"), ("c", "3"))], [1, 1]
        )
        merge_lock.release("agent-a", root=self.a)
        # The releaser cannot chain a second merge past the waiters (#1859 -> #1865).
        self.assertEqual(merge_lock.acquire("agent-a", "105", root=self.a), 1)
        self.assertEqual(self.queue(), ["b", "c", "agent-a"])
        self.assertEqual(merge_lock.acquire("c", "3", root=self.b), 1)  # the free lock is reserved for b
        self.assertEqual(merge_lock.acquire("b", "2", root=self.b), 0)
        self.assertEqual(self.queue(), ["c", "agent-a"])
        merge_lock.release("b", root=self.b)
        self.assertEqual(merge_lock.acquire("c", "3", root=self.b), 0)

    def test_a_renewal_keeps_the_queue_order(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        for holder in ("b", "c"):
            merge_lock.acquire(holder, "2", root=self.b)
        self.tick(5)
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 0)
        lock = self.held_lock()
        self.assertEqual((lock.renewed, [w.id for w in lock.waiters]), (self.now, ["b", "c"]))

    def test_waiter_refreshes_are_throttled_and_a_silent_waiter_expires(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        merge_lock.acquire("b", "2", root=self.b)
        sha = self.held_lock().sha
        self.tick(1)
        merge_lock.acquire("b", "2", root=self.b)
        self.assertEqual(self.held_lock().sha, sha)  # no push within 3 min
        self.tick(2)
        merge_lock.acquire("b", "2", root=self.b)
        self.assertEqual(self.held_lock().waiters[0].seen, self.now)
        self.tick(merge_lock.WAITER_TTL / 60 + 1)
        merge_lock.acquire("c", "3", root=self.b)
        self.assertEqual(self.queue(), ["c"])

    def test_a_head_is_dropped_when_it_does_not_claim_or_its_pr_finished(self) -> None:
        for finished, minutes, expected in ((set(), 1, 1), (set(), 3, 0), ({"2"}, 0, 0)):
            with self.subTest(finished=finished, minutes=minutes):
                self.put(merge_lock.Lock("", "agent-a", "101", renewed=self.now))
                for holder, pr in (("b", "2"), ("c", "3")):
                    merge_lock.acquire(holder, pr, root=self.b)
                merge_lock.release("agent-a", root=self.a)
                self.finished, start = finished, self.now
                self.tick(minutes)
                self.assertEqual(merge_lock.acquire("c", "3", root=self.b), expected)
                self.now = start

    def test_a_train_lock_is_finished_only_when_its_prs_all_are(self) -> None:
        cases = (
            (("1", "2"), {"1"}, 0, 1),
            (("1", "2"), {"1", "2"}, 0, 0),
            ((), {"train"}, 0, 1),
            ((), set(), 31, 0),
        )
        for prs, finished, minutes, expected in cases:
            with self.subTest(prs=prs, finished=finished, minutes=minutes):
                self.put(merge_lock.Lock("", "run-7", "train", prs, renewed=self.now))
                self.finished, start = finished, self.now
                self.tick(minutes)
                self.assertEqual(merge_lock.acquire("b", "9", root=self.b), expected)
                self.now = start

    def test_a_renewal_or_release_that_loses_to_a_waiter_refresh_retries(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        before = self.held_lock()
        merge_lock.acquire("b", "2", root=self.b)
        with self.first_read(before):
            self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), 0)
        before = self.held_lock()
        merge_lock.acquire("c", "3", root=self.b)
        with self.first_read(before):
            self.assertEqual(merge_lock.release("agent-a", root=self.a), 0)
        lock = self.held_lock()
        self.assertEqual((lock.free, [w.id for w in lock.waiters]), (True, ["b", "c"]))

    def test_each_pr_is_read_at_most_once_per_acquire(self) -> None:
        reads: list[str] = []
        with mock.patch.object(merge_lock, "pr_finished", lambda pr, root=None: reads.append(pr) or False):
            merge_lock.acquire("agent-a", "101", root=self.a)
            before = self.held_lock()
            merge_lock.acquire("c", "303", root=self.b)
            reads.clear()
            with self.first_read(before):  # a stale first read: the write loses and is retried
                merge_lock.acquire("b", "202", root=self.b)
            self.assertEqual(reads, ["202", "101"])  # held: the holder's PR, once
            merge_lock.release("agent-a", root=self.a)
            for holder, pr, expected in (("b", "202", ["202", "303"]), ("c", "303", ["303"])):
                reads.clear()
                merge_lock.acquire(holder, pr, root=self.b)
                self.assertEqual(reads, expected)  # reserved: the head's PR, unless the caller is the head

    def test_a_renewal_after_a_takeover_exits_4(self) -> None:
        merge_lock.acquire("agent-a", "101", root=self.a)
        before = self.held_lock()
        self.tick(31)
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 0)
        with self.first_read(before):  # taken over while this renewal ran
            self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a), merge_lock.LOST)
        # Taken over between two calls: `--renew` says the caller held it.
        self.assertEqual(merge_lock.acquire("agent-a", "101", root=self.a, renew=True), merge_lock.LOST)
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b, renew=True), 0)
        self.assertEqual(self.queue(), [])

    def test_bad_values_are_refused_on_write(self) -> None:
        bad = [("a\nstate: free", "1", None), ("a", "1\nstate: free", None), ("unknown", "1", None)]
        bad += [("a", "0", None), ("a", "1", ["2"]), ("a", "train", ["1"] * 6), ("a b", "1", None)]
        for holder, pr, prs in bad:
            with self.subTest(holder=holder, pr=pr), self.assertRaises(merge_lock.LockFormatError):
                merge_lock.acquire(holder, pr, root=self.a, prs=prs)
        self.assertIsNone(self.lock())
        for waiters in ([merge_lock.Waiter("x\ny", "1", 0, 0)], [merge_lock.Waiter("w", "1", 0, 0)] * 21):
            with self.assertRaises(merge_lock.LockFormatError):
                merge_lock.render(merge_lock.Lock("", None, waiters=tuple(waiters)))
        with mock.patch("sys.stderr"):
            self.assertEqual(merge_lock.main(["acquire", "--pr", "1", "--holder", "a\nstate: free"]), 2)

    def test_parse_drops_malformed_lines_and_fails_closed(self) -> None:
        w = merge_lock.Waiter
        cases = {
            "holder: a b\npr: 1": ("unknown", "1", 1000, []),
            "pr: 1": ("unknown", "1", 1000, []),
            "holder: a\nstate: free\npr: 1": ("a", "1", 1000, []),
            "holder: a\npr: 1; rm\nrenewed: x": ("a", "", 1000, []),
            "holder: a\npr: 1\nrenewed: 99999": ("a", "1", 1000, []),
            "holder: a\npr: 1\nrenewed: 1050": ("a", "1", 1050, []),
            "holder: a\npr: 1\nrenewed: 900\nwaiter: ok 1 900 950\nwaiter: bad! 1 1 1\nwaiter: ok 2 1 1\n"
            "waiter: fut 1 900 99999\nwaiter: short 1 2": (
                "a",
                "1",
                900,
                [w("ok", "1", 900, 950), w("fut", "1", 900, 0)],
            ),
        }
        for body, (holder, pr, renewed, waiters) in cases.items():
            with self.subTest(body=body):
                lock = merge_lock.parse_lock("s", 1000, f"merge-lock\n\nversion: 2\n{body}\n")
                self.assertEqual(
                    (lock.holder, lock.pr, lock.renewed, list(lock.waiters)), (holder, pr, renewed, waiters)
                )
        train = merge_lock.parse_lock(
            "s", 1000, "merge-lock\n\nversion: 2\nholder: m\npr: train\nprs: 1 2 3 4 5 6\n"
        )
        free = merge_lock.parse_lock("s", 1000, "merge-lock\n\nversion: 2\nstate: free\nfreed: 99999\n")
        self.assertEqual((train.prs, free.free, free.freed), ((), True, 1000))

    def test_version_1_commits_parse(self) -> None:
        v1 = merge_lock.parse_lock(
            "s", 1000, "merge-lock\n\nholder: old\npr: 7\nrenewed: 5\nwaiter: x 7 1 1\n"
        )
        self.assertEqual((v1.holder, v1.pr, v1.renewed, v1.waiters), ("old", "7", 1000, ()))
        self.assertTrue(merge_lock.parse_lock("s", 1000, "merge-lock\n\nstate: free\n").free)
        self.push("merge-lock\n\nholder: old\npr: 7\n")
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 1)
        lock = self.held_lock()
        self.assertEqual((lock.holder, [w.id for w in lock.waiters]), ("old", ["agent-b"]))

    def test_future_timestamps_clamp_to_the_committer_time_not_the_read_time(self) -> None:
        far = "9999999999"
        self.push(
            f"merge-lock\n\nversion: 2\nholder: x\npr: 1\nrenewed: {far}\nwaiter: w 5 {far} {far}\n",
            "2000-01-01T00:00:00Z",
        )
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 0)  # stale, and w expired

    def test_a_missing_holder_reads_as_held_until_the_ttl(self) -> None:
        self.push("merge-lock\n\nversion: 2\npr: 5\n")
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 1)
        self.tick(31 + 60)  # the fake clock starts an hour before the commit time
        self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 0)

    def test_the_queue_is_capped(self) -> None:
        full = tuple(merge_lock.Waiter(f"w{i}", "1", self.now, self.now) for i in range(merge_lock.QUEUE_MAX))
        self.put(merge_lock.Lock("", "agent-a", "101", renewed=self.now, waiters=full))
        with mock.patch("builtins.print") as printed:
            self.assertEqual(merge_lock.acquire("agent-b", "202", root=self.b), 1)
        self.assertIn("queue full", printed.call_args.args[0])
        self.assertNotIn("agent-b", self.queue())

    def test_main_takes_a_train_and_replaces_its_prs(self) -> None:
        self.assertEqual(merge_lock.main(["acquire", "--train", "run-7", "--prs", "1", "2"], self.a), 0)
        self.assertEqual(merge_lock.main(["acquire", "--train", "run-7"], self.a), 0)
        self.assertEqual(self.held_lock().prs, ("1", "2"))
        self.assertEqual(merge_lock.main(["acquire", "--train", "run-7", "--prs"], self.a), 0)
        lock = self.held_lock()
        self.assertEqual((lock.holder, lock.pr, lock.prs), ("run-7", "train", ()))

    def test_main_needs_a_holder_and_a_numeric_pr(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=True), mock.patch("sys.stderr"):
            self.assertEqual(merge_lock.main(["acquire", "--pr", "1"]), 2)
        with self.assertRaises(SystemExit), mock.patch("sys.stderr"):
            merge_lock.main(["acquire", "--pr", "$(whoami)", "--holder", "x"])


class PrFinished(unittest.TestCase):
    """`pr_finished` over a stubbed API (no network)."""

    def read(self, payload: object, url: str = "https://github.com/o/r.git") -> bool | None:
        class Response:
            def __enter__(self) -> "Response":
                return self

            def __exit__(self, *exc: object) -> None:
                return None

            def read(self, *_: object) -> bytes:
                return json.dumps(payload).encode()

        remote = subprocess.CompletedProcess([], 0, stdout=url + "\n")
        with (
            mock.patch.object(merge_lock, "git", return_value=remote),
            mock.patch.object(merge_lock.urllib.request, "urlopen", return_value=Response()),
        ):
            return merge_lock.pr_finished("7")

    def test_open_merged_and_closed(self) -> None:
        self.assertIs(self.read({"state": "open", "merged": False}), False)
        self.assertIs(self.read({"state": "closed", "merged": True}), True)
        self.assertIs(self.read({"state": "closed", "merged": False}), True)

    def test_an_unreadable_state_is_unknown(self) -> None:
        self.assertIsNone(self.read({"message": "Not Found"}))
        self.assertIsNone(self.read({"state": "open"}, url="git@example.com:o/r.git"))
        with (
            mock.patch.object(merge_lock, "git", return_value=subprocess.CompletedProcess([], 0, "https://github.com/o/r\n")),
            mock.patch.object(merge_lock.urllib.request, "urlopen", side_effect=OSError("offline")),
        ):
            self.assertIsNone(merge_lock.pr_finished("7"))


if __name__ == "__main__":
    unittest.main()
