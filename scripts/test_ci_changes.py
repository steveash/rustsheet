from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import check_docs_readers as readers  # noqa: E402 (import after sys.path setup)
import ci_changes  # noqa: E402


def classify(*paths: str) -> bool:
    return ci_changes.classify(b"\0".join(p.encode() for p in paths) + (b"\0" if paths else b""))


class Classifier(unittest.TestCase):
    def test_docs_and_claude_files_alone_are_docs_only(self) -> None:
        self.assertFalse(classify("docs/tasks/M2/T-1.md", "docs/status.md", ".claude/skills/x/SKILL.md"))

    def test_excel_semantics_is_code(self) -> None:
        # The eval harness validates DEVIATION anchors in it (decision 0049).
        self.assertTrue(classify("docs/excel-semantics/text.md"))
        self.assertTrue(classify("docs/status.md", "docs/excel-semantics/arrays.md"))

    def test_progress_report_inputs_are_code(self) -> None:
        # The progress report generator reads them (SPEC-0022, decision 0120).
        self.assertTrue(classify("docs/progress/history.json"))
        self.assertTrue(classify("docs/progress.md"))
        self.assertTrue(classify("docs/milestones.md", "docs/status.md"))
        self.assertTrue(classify("docs/progress/functions.md"))
        # Their neighbours stay docs-only: a file entry matches only itself (decision 0120).
        self.assertFalse(classify("docs/parity.md", "docs/status.md", "docs/research-plan.md"))
        self.assertFalse(classify("docs/progress.md.bak", "docs/progress.mdx", "docs/milestones.md~"))

    def test_rename_of_crate_file_into_docs_is_code(self) -> None:
        # --no-renames lists both sides, so the crate side makes it code.
        self.assertTrue(classify("crates/rustsheet-engine/src/lib.rs", "docs/lib.rs"))

    def test_top_level_markdown_is_docs_only(self) -> None:
        self.assertFalse(classify("CLAUDE.md"))
        self.assertFalse(classify("README.md", "docs/status.md"))

    def test_github_and_dotfiles_are_code(self) -> None:
        self.assertTrue(classify(".github/workflows/ci.yml"))
        self.assertTrue(classify(".gitignore"))
        self.assertTrue(classify("docs/status.md", ".typos.toml"))

    def test_nested_markdown_outside_docs_is_code(self) -> None:
        self.assertTrue(classify("crates/rustsheet-engine/README.md"))

    def test_empty_or_undecodable_input_is_code(self) -> None:
        self.assertTrue(ci_changes.classify(b""))
        self.assertTrue(ci_changes.classify(b"\0\0"))
        self.assertTrue(ci_changes.classify(b"docs/\xff.md\0"))

    def test_file_names_with_newlines_stay_intact(self) -> None:
        # One docs file whose name holds a newline, not a docs file and a crate file.
        self.assertFalse(classify("docs/a\ncrates/x.rs"))

    def test_cli_prints_one_output_line(self) -> None:
        run = subprocess.run(
            [sys.executable, str(HERE / "ci_changes.py")],
            input=b"docs/status.md\0",
            capture_output=True,
            check=True,
        )
        self.assertEqual(run.stdout, b"code=false\n")


def git(repo: Path, *args: str) -> str:
    env = {
        **os.environ,
        "GIT_AUTHOR_NAME": "t",
        "GIT_AUTHOR_EMAIL": "t@example.com",
        "GIT_COMMITTER_NAME": "t",
        "GIT_COMMITTER_EMAIL": "t@example.com",
        "GIT_CONFIG_GLOBAL": os.devnull,
        "GIT_CONFIG_NOSYSTEM": "1",
    }
    run = subprocess.run(["git", *args], cwd=repo, env=env, capture_output=True, text=True, check=True)
    return run.stdout.strip()


def commit(repo: Path, *paths: str) -> str:
    """Commits a change to each of `paths` and returns the new sha."""
    for p in paths:
        f = repo / p
        f.parent.mkdir(parents=True, exist_ok=True)
        with f.open("a") as out:
            out.write("x\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "c")
    return git(repo, "rev-parse", "HEAD")


class MergeBase(unittest.TestCase):
    """`--merge-base A B` diffs the single merge base of A and B to B (decision 0147)."""

    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.repo = Path(tmp.name) / "r"
        self.repo.mkdir()
        git(self.repo, "init", "-q", "-b", "main")
        self.m0 = commit(self.repo, "crates/a.rs", "docs/a.md")

    def code(self, a: str, b: str, repo: Path | None = None) -> bool:
        return ci_changes.classify_range(a, b, cwd=repo or self.repo)

    def branch(self, name: str, start: str) -> None:
        git(self.repo, "checkout", "-q", "-b", name, start)

    def test_train_of_docs_prs_on_main_is_docs_only(self) -> None:
        # A train starts at M0 = main's tip and merges PRs onto it; main has not moved.
        self.branch("pr1", self.m0)
        pr1 = commit(self.repo, "docs/tasks/T-1.md")
        self.branch("merge-train/current", self.m0)
        git(self.repo, "merge", "-q", "--no-ff", "--no-edit", pr1)
        commit(self.repo, "docs/status.md")
        self.assertFalse(self.code("main", "HEAD"))
        self.assertEqual(self.cli("main", "HEAD"), b"code=false\n")

    def test_train_with_a_code_pr_is_code(self) -> None:
        self.branch("merge-train/current", self.m0)
        commit(self.repo, "docs/status.md")
        commit(self.repo, "crates/b.rs")
        self.assertTrue(self.code("main", "HEAD"))

    def test_train_base_ignores_main_moving_on(self) -> None:
        # main gains a code commit after the train was built: only the train's own diff counts.
        self.branch("merge-train/current", self.m0)
        commit(self.repo, "docs/status.md")
        git(self.repo, "checkout", "-q", "main")
        commit(self.repo, "crates/a.rs")
        self.assertFalse(self.code("main", "merge-train/current"))

    def test_docs_only_pr_behind_main_is_docs_only(self) -> None:
        # Run 40622986699 (PR #2119): base.sha lagged main, and main had gained code since the
        # branch point. The old diff (base.sha to GitHub's test merge) listed main's code files.
        self.branch("pr", self.m0)
        pr_head = commit(self.repo, "docs/specs/SPEC-1.md")
        git(self.repo, "checkout", "-q", "main")
        lagging_base = commit(self.repo, "crates/b.rs")
        tip = commit(self.repo, "crates/c.rs")
        git(self.repo, "checkout", "-q", "--detach", tip)
        git(self.repo, "merge", "-q", "--no-ff", "--no-edit", pr_head)
        test_merge = git(self.repo, "rev-parse", "HEAD")
        old = git(self.repo, "diff", "--name-only", lagging_base, test_merge).split("\n")
        self.assertTrue(classify(*old))
        self.assertFalse(self.code(lagging_base, pr_head))
        self.assertFalse(self.code(tip, pr_head))

    def test_code_pr_behind_main_is_code(self) -> None:
        self.branch("pr", self.m0)
        pr_head = commit(self.repo, "docs/a.md", "scripts/x.py")
        git(self.repo, "checkout", "-q", "main")
        base = commit(self.repo, "docs/b.md")
        self.assertTrue(self.code(base, pr_head))

    def test_several_merge_bases_are_code(self) -> None:
        # A criss-cross history: x and y each merged the other's first commit.
        self.branch("x", self.m0)
        x1 = commit(self.repo, "docs/x.md")
        self.branch("y", self.m0)
        y1 = commit(self.repo, "docs/y.md")
        git(self.repo, "checkout", "-q", "x")
        git(self.repo, "merge", "-q", "--no-ff", "--no-edit", y1)
        x2 = commit(self.repo, "docs/x2.md")
        git(self.repo, "checkout", "-q", "y")
        git(self.repo, "merge", "-q", "--no-ff", "--no-edit", x1)
        y2 = commit(self.repo, "docs/y2.md")
        self.assertEqual(len(git(self.repo, "merge-base", "--all", x2, y2).split()), 2)
        self.assertTrue(self.code(x2, y2))
        self.assertEqual(self.cli(x2, y2), b"code=true\n")

    def test_failed_merge_base_is_code(self) -> None:
        # Unrelated histories have no merge base; a missing or malformed revision fails.
        git(self.repo, "checkout", "-q", "--orphan", "other")
        other = commit(self.repo, "docs/o.md")
        self.assertTrue(self.code(self.m0, other))
        self.assertTrue(self.code("0" * 40, self.m0))
        self.assertTrue(self.code("origin/main", "HEAD"))
        for bad in ("", "--all", "-h", "a b", "HEAD\nHEAD"):
            self.assertTrue(self.code(bad, self.m0), bad)
            self.assertTrue(self.code(self.m0, bad), bad)
        self.assertTrue(self.code(self.m0, self.m0, repo=self.repo.parent))

    def test_same_commit_is_code(self) -> None:
        # Nothing changed: no paths, so run everything (an empty diff is never docs-only).
        self.assertTrue(self.code(self.m0, self.m0))

    def test_shallow_clone_is_code(self) -> None:
        # A shallow clone can hide the true merge base, so it must never yield docs-only.
        self.branch("pr", self.m0)
        pr_head = commit(self.repo, "docs/a.md")
        shallow = self.repo.parent / "s"
        git(self.repo.parent, "clone", "-q", "--depth", "2", "--branch", "pr", self.repo.as_uri(), str(shallow))
        # Both commits are in the shallow clone and their merge base is right, yet doubt runs CI.
        self.assertEqual(git(shallow, "merge-base", self.m0, pr_head), self.m0)
        self.assertTrue(self.code(self.m0, pr_head, repo=shallow))
        self.assertFalse(self.code(self.m0, pr_head))

    def test_cli_without_arguments_still_reads_stdin(self) -> None:
        # An older workflow (or 0066's stdin contract) keeps working.
        self.assertEqual(self.cli_raw([], b"docs/a.md\0"), b"code=false\n")

    def test_cli_with_bad_arguments_is_code(self) -> None:
        self.assertEqual(self.cli_raw(["--merge-base", "HEAD"], b""), b"code=true\n")
        self.assertEqual(self.cli_raw(["--nonsense"], b"docs/a.md\0"), b"code=true\n")

    def cli(self, a: str, b: str) -> bytes:
        return self.cli_raw(["--merge-base", a, b], b"")

    def cli_raw(self, args: list[str], stdin: bytes) -> bytes:
        run = subprocess.run(
            [sys.executable, "-I", str(HERE / "ci_changes.py"), *args],
            input=stdin,
            cwd=self.repo,
            capture_output=True,
            check=True,
        )
        return run.stdout


class DocsReaders(unittest.TestCase):
    def test_literal_naming_skipped_docs_path_fails(self) -> None:
        for line in [
            'let s = include_str!("../../../docs/tasks/README.md");',
            "root = Path(ROOT) / 'docs'",
            'fs::read(".claude/skills/x.md")',
            'let p = root.join("docs/status.md");',
            'page = "docs/progress.mdx"',
        ]:
            self.assertEqual(len(readers.violations("crates/x/src/a.rs", line)), 1, line)

    def test_code_read_paths_and_prose_pass(self) -> None:
        for line in [
            'const SEMANTICS_DIR: &str = "docs/excel-semantics/";',
            'root.join("docs/excel-semantics")',
            'HISTORY = Path("docs/progress/history.json")',
            'repo / "docs/milestones.md"',
            "page = '../docs/progress.md'",
            '// see "docs/status.md"',
            '# reads "docs/status.md"',
            'return False, "requires Excel (see docs/architecture/oracle.md)"',
            'dir: "tests/docs",',
        ]:
            self.assertEqual(readers.violations("crates/x/src/a.rs", line), [], line)

    def test_listed_crate_docs_dir_passes(self) -> None:
        line = 'EXAMPLES_DIR="docs/examples"'
        self.assertEqual(readers.violations("bindings/python/run_examples.sh", line), [])
        self.assertEqual(len(readers.violations("bindings/python/other.sh", line)), 1)


if __name__ == "__main__":
    unittest.main()
