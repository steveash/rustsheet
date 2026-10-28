"""Tests for scripts/merge_train.py (SPEC-0027 section 4 and its Test plan), against temporary
remotes and worktrees (stdlib unittest; run by `make hygiene`)."""

from __future__ import annotations

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


if __name__ == "__main__":
    unittest.main()
