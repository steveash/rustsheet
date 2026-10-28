"""Tests for scripts/merge_train.py (SPEC-0027 section 4 and its Test plan), against temporary
remotes and worktrees (stdlib unittest; run by `make hygiene`)."""

from __future__ import annotations

import random
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_train  # noqa: E402 (the sys.path line above must run first)

SCRIPTS = Path(__file__).resolve().parent
TASK = """---
id: {tid}
title: "t"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---
"""


def run(*args: str, cwd: Path) -> str:
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


class Mode(unittest.TestCase):
    def test_the_read_fails_closed(self) -> None:
        self.assertEqual(merge_train.parse_mode('mode = "train"\n'), merge_train.TRAIN)
        self.assertEqual(merge_train.parse_mode('mode = "shadow"\n'), merge_train.SHADOW)
        for text in (
            None,  # missing file
            "mode = ",  # malformed
            'mode = "train"\nmode = "train"\n',  # duplicate key
            'mode = "fast"\n',  # unknown value
            "mode = 1\n",
            '[x]\nmode = "train"\n',
            "",
        ):
            with self.subTest(text=text):
                self.assertEqual(merge_train.parse_mode(text), merge_train.LOCK)

    def test_the_mode_comes_from_origin_main_not_the_checkout(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            origin, clone = base / "origin.git", base / "clone"
            run("git", "init", "-q", "--bare", "-b", "main", str(origin), cwd=base)
            run("git", "clone", "-q", str(origin), str(clone), cwd=base)
            for key, value in (("user.name", "t"), ("user.email", "t@invalid")):
                run("git", "config", key, value, cwd=clone)
            run("git", "checkout", "-q", "-b", "main", cwd=clone)
            (clone / "scripts").mkdir()
            (clone / "README").write_text("x")
            run("git", "add", "-A", cwd=clone)
            run("git", "commit", "-q", "-m", "no switch", cwd=clone)
            run("git", "push", "-q", "origin", "main", cwd=clone)
            # The checkout says train; origin/main has no switch file.
            (clone / merge_train.SWITCH).write_text('mode = "train"\n')
            self.assertEqual(merge_train.read_mode(clone), merge_train.LOCK)
            run("git", "add", "-A", cwd=clone)
            run("git", "commit", "-q", "-m", "switch", cwd=clone)
            run("git", "push", "-q", "origin", "main", cwd=clone)
            self.assertEqual(merge_train.read_mode(clone), merge_train.TRAIN)
            self.assertEqual(merge_train.read_mode(base / "nowhere"), merge_train.LOCK)

    def test_the_switch_file_says_lock(self) -> None:
        self.assertEqual(merge_train.parse_mode((SCRIPTS / "merge_train.toml").read_text()), merge_train.LOCK)


class IdChecks(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        # M0's scripts, and a train tree in a git repository of its own.
        self.scripts = base / "m0" / "scripts"
        self.scripts.mkdir(parents=True)
        for name in ("tasks.py", "check_decisions.py"):
            shutil.copy(SCRIPTS / name, self.scripts / name)
        self.tree = base / "train"
        for path, text in (
            ("docs/tasks/M2/T-0001-a.md", TASK.format(tid="T-0001")),
            ("docs/decisions/README.md", "x"),
            ("docs/decisions/0001-a.md", "x"),
        ):
            (self.tree / path).parent.mkdir(parents=True, exist_ok=True)
            (self.tree / path).write_text(text)
        run("git", "init", "-q", str(self.tree), cwd=base)
        run("git", "add", "-A", cwd=self.tree)

    def add(self, path: str, text: str) -> None:
        (self.tree / path).parent.mkdir(parents=True, exist_ok=True)
        (self.tree / path).write_text(text)
        run("git", "add", "-A", cwd=self.tree)

    def test_a_clean_tree_passes_and_collisions_fail(self) -> None:
        self.assertEqual(merge_train.id_check_failures(self.scripts, self.tree), [])
        self.add("docs/tasks/M2/T-0001-b.md", TASK.format(tid="T-0001"))
        self.add("docs/decisions/0001-b.md", "x")
        failures = merge_train.id_check_failures(self.scripts, self.tree)
        self.assertTrue(any("duplicate id T-0001" in f for f in failures), failures)
        self.assertTrue(any("decision number 0001 used twice" in f for f in failures), failures)

    def test_a_planted_module_in_the_train_tree_is_not_imported(self) -> None:
        planted = self.tree / "imported"
        hostile = f"from pathlib import Path\nPath({str(planted)!r}).write_text('x')\n"
        for name in ("tasks.py", "check_decisions.py", "re.py", "pathlib.py", "sitecustomize.py"):
            self.add(f"scripts/{name}", hostile)
            self.add(name, hostile)
        self.assertEqual(merge_train.id_check_failures(self.scripts, self.tree), [])
        self.assertFalse(planted.exists())

    def test_root_is_refused_outside_validate(self) -> None:
        tasks = subprocess.run(
            [sys.executable, "-I", str(self.scripts / "tasks.py"), "list", "--root", str(self.tree)],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(tasks.returncode, 2)
        decisions = subprocess.run(
            [sys.executable, "-I", str(self.scripts / "check_decisions.py"), "--root", str(self.tree / "none")],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(decisions.returncode, 2)


class Build(unittest.TestCase):
    """Steps 3-4 on a scratch repository whose `main` is M0; PRs are branches of it."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name) / "repo"
        run("git", "init", "-q", "-b", "main", str(self.repo), cwd=Path(self.tmp.name))
        for key, value in (("user.name", "t"), ("user.email", "t@invalid")):
            self.git("config", key, value)
        self.write("docs/decisions/README.md", "x")
        self.write("docs/decisions/0001-a.md", "x")
        self.write("shared.txt", "one\ntwo\nthree\n")
        self.m0 = self.commit("M0")

    def git(self, *args: str) -> str:
        return run("git", *args, cwd=self.repo)

    def write(self, path: str, text: str) -> None:
        (self.repo / path).parent.mkdir(parents=True, exist_ok=True)
        (self.repo / path).write_text(text)

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", message)
        return self.git("rev-parse", "HEAD")

    def pr(self, start: str, files: dict[str, str | None]) -> str:
        """A PR head: from `start`, write each file (None deletes it)."""
        self.git("checkout", "-q", "-f", "--detach", start)
        for path, text in files.items():
            if text is None:
                (self.repo / path).unlink()
            else:
                self.write(path, text)
        return self.commit(f"pr {sorted(files)}")

    def build(self, *heads: str, base: str | None = None) -> merge_train.Train:
        return merge_train.build(self.repo, SCRIPTS, base or self.m0, list(enumerate(heads, 1)))

    def test_independent_prs_merge_in_order_up_to_the_cap(self) -> None:
        heads = [self.pr(self.m0, {f"f{k}.txt": "x"}) for k in range(merge_train.BATCH_MAX + 1)]
        train = self.build(*heads)
        self.assertEqual([m.pr for m in train.members], list(range(1, merge_train.BATCH_MAX + 1)))
        self.assertEqual((train.deferred, train.ejected), ([], []))
        self.assertEqual(self.git("rev-parse", f"{train.tip}^1"), train.members[-2].merge)

    def test_stacked_and_intra_batch_conflicts_defer_and_main_conflicts_eject(self) -> None:
        first = self.pr(self.m0, {"a.txt": "a", "shared.txt": "one\nTWO\nthree\n"})
        stacked = self.pr(first, {"b.txt": "b"})
        rival = self.pr(self.m0, {"shared.txt": "one\n2\nthree\n"})
        self.git("checkout", "-q", "-f", "--detach", self.m0)
        self.write("shared.txt", "one\nzwei\nthree\n")
        main = self.commit("main moves")
        train = self.build(first, stacked, rival, base=main)
        self.assertEqual([m.pr for m in train.members], [])
        # The stacked PR carries the first one's edit, so it conflicts with main too.
        self.assertEqual([pr for pr, _ in train.ejected], [1, 2, 3])
        train = self.build(first, stacked, rival)
        self.assertEqual([m.pr for m in train.members], [1])
        self.assertEqual([pr for pr, _ in train.deferred], [2, 3])

    def test_a_criss_cross_history_defers(self) -> None:
        x = self.pr(self.m0, {"x.txt": "x"})
        y = self.pr(self.m0, {"y.txt": "y"})
        self.git("checkout", "-q", "-f", "--detach", x)
        self.git("merge", "-q", "--no-edit", y)
        one = self.pr(self.git("rev-parse", "HEAD"), {"one.txt": "1"})
        self.git("checkout", "-q", "-f", "--detach", y)
        self.git("merge", "-q", "--no-edit", x)
        two = self.pr(self.git("rev-parse", "HEAD"), {"two.txt": "2"})
        self.assertEqual(len(self.git("merge-base", "--all", one, two).split()), 2)
        train = self.build(one, two)
        self.assertEqual(([m.pr for m in train.members], [pr for pr, _ in train.deferred]), ([1], [2]))

    def test_id_collisions_eject_the_later_copy(self) -> None:
        task = TASK.format(tid="T-0005")
        first = self.pr(self.m0, {"docs/tasks/M2/T-0005-a.md": task})
        later = self.pr(self.m0, {"docs/tasks/M2/T-0005-b.md": task})
        with_main = self.pr(self.m0, {"docs/decisions/0001-b.md": "x"})
        train = self.build(first, later, with_main)
        self.assertEqual([m.pr for m in train.members], [1])
        self.assertEqual(
            train.ejected,
            [
                (2, "id collision with T-0005; renumber per decision 0121 (2)"),
                (3, "id collision with 0001; renumber per decision 0121 (2)"),
            ],
        )

    def test_squashing_the_members_in_order_gives_each_train_tree(self) -> None:
        """SPEC-0027 section 5: for independent PRs, each squash onto the previous one has the
        train's tree, with renames, deletions and merges of `main` into a PR included."""
        rng = random.Random(2987)
        for trial in range(3):
            with self.subTest(trial=trial):
                self.git("checkout", "-q", "-f", "--detach", self.m0)
                history = [self.m0]
                for i in range(3):
                    self.write(f"main{trial}-{i}.txt", f"{i}\n")
                    history.append(self.commit(f"main {i}"))
                base = history[-1]
                heads = []
                for k in range(merge_train.BATCH_MAX):
                    own = f"pr{trial}-{k}"
                    head = self.pr(rng.choice(history[:-1]), {f"{own}.txt": "a\n", f"{own}-gone.txt": "b\n"})
                    if rng.random() < 0.5:  # a later merge of main into the PR
                        self.git("merge", "-q", "--no-edit", rng.choice(history))
                    self.git("mv", f"{own}.txt", f"{own}-renamed.txt")
                    heads.append(self.pr(self.commit("rename"), {f"{own}-gone.txt": None}))
                train = self.build(*heads, base=base)
                self.assertEqual(len(train.members), merge_train.BATCH_MAX)
                tip = set(self.git("ls-tree", "-r", "--name-only", train.tip).split())
                self.assertIn(f"pr{trial}-0-renamed.txt", tip)
                self.assertNotIn(f"pr{trial}-0-gone.txt", tip)
                squash = base
                for member in train.members:
                    tree = self.git("merge-tree", "--write-tree", squash, member.head)
                    squash = self.git("commit-tree", tree, "-p", squash, "-m", "squash")
                    self.assertEqual(tree, self.git("rev-parse", f"{member.merge}^{{tree}}"))


if __name__ == "__main__":
    unittest.main()
