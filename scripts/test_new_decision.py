"""Unit tests for scripts/new_decision.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_decisions  # noqa: E402 (the sys.path line above must run first)
import new_decision  # noqa: E402
import tasks  # noqa: E402


class NewDecision(unittest.TestCase):
    def run_new(self, title: str, on_branches: set[str] | Exception) -> tuple[int, list[str]]:
        """Runs `new_decision` in a scratch tree holding 0001 and 0002; `on_branches` is what the
        origin branch scan returns, or the error `git` raises."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            decisions = root / "docs" / "decisions"
            decisions.mkdir(parents=True)
            for name in ("0001-a.md", "0002-b.md", "README.md"):
                (decisions / name).write_text("x\n")
            run = mock.Mock(side_effect=on_branches) if isinstance(on_branches, Exception) else None
            with mock.patch.object(tasks, "ROOT", root), \
                    mock.patch.object(check_decisions, "DECISIONS", decisions), \
                    mock.patch.object(tasks.subprocess, "run", run or self.fake_git(on_branches)):
                code = new_decision.new_decision(title, fetch=False)
            return code, sorted(p.name for p in decisions.glob("*.md"))

    @staticmethod
    def fake_git(numbers: set[str]) -> mock.Mock:
        listing = "".join(f"docs/decisions/{n}-other.md\n" for n in sorted(numbers)) + "docs/decisions/README.md\n"

        def run(cmd: list[str], **_: object) -> mock.Mock:
            if cmd[3] == "for-each-ref":
                return mock.Mock(stdout="refs/remotes/origin/agent/x\n")
            assert cmd[-1] == "docs/decisions", cmd
            return mock.Mock(stdout=listing)

        return mock.Mock(side_effect=run)

    def test_a_number_taken_only_on_a_pushed_branch_is_skipped(self) -> None:
        code, names = self.run_new("Share the allocator", {"0003", "0004"})
        self.assertEqual(code, 0)
        self.assertIn("0005-share-the-allocator.md", names)

    def test_without_git_the_working_tree_decides(self) -> None:
        code, names = self.run_new("Offline", OSError("no git"))
        self.assertEqual(code, 0)
        self.assertIn("0003-offline.md", names)

    def test_names_pass_the_record_check(self) -> None:
        for title in ("Ünïcode & Punctuation: Here!", "!!!", "A very long title with many more than eight words in it"):
            _, names = self.run_new(title, set())
            self.assertEqual(check_decisions.problems(names), [], title)

    def test_template_has_the_record_sections(self) -> None:
        for section in ("Status", "Context", "Decision", "Consequences"):
            self.assertIn(f"- **{section}:**", new_decision.TEMPLATE)

    def test_multi_line_title_is_refused(self) -> None:
        code, names = self.run_new("a\nb", set())
        self.assertEqual(code, 2)
        self.assertEqual(names, ["0001-a.md", "0002-b.md", "README.md"])


if __name__ == "__main__":
    unittest.main()
