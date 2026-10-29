"""Smoke test: `make new-task` / `make new-decision` pass a TITLE with shell metacharacters through
unchanged (T-0084). `PY` is replaced by `printf`, so the recipe only echoes the arguments."""

from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TITLE = 'a "quoted" ;;; title with `tick` and \'single\' and ()'


def recipe_args(target: str, *extra: str) -> list[str]:
    out = subprocess.run(
        ["make", "-s", target, "PY=printf '%s\\n'", f"TITLE={TITLE}", *extra],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    return out.splitlines()


class MakefileTitles(unittest.TestCase):
    def test_new_task_title_is_unchanged(self) -> None:
        args = recipe_args("new-task", "MILESTONE=M0", "LANE=harness", "SIZE=S")
        self.assertEqual(args[args.index("--title") + 1], TITLE)
        self.assertEqual(args[args.index("--lane") + 1], "harness")

    def test_new_decision_title_is_unchanged(self) -> None:
        args = recipe_args("new-decision")
        self.assertEqual(args[args.index("--title") + 1], TITLE)

    def test_dollar_needs_make_escape(self) -> None:
        # make itself expands a single `$` on its command line; `$$` is the documented way to write one.
        out = subprocess.run(
            ["make", "-s", "new-decision", "PY=printf '%s\\n'", "TITLE=cost $$5 $$(x)"],
            cwd=ROOT, capture_output=True, text=True, check=True,
        ).stdout.splitlines()
        self.assertEqual(out[-1], "cost $5 $(x)")


if __name__ == "__main__":
    unittest.main()
