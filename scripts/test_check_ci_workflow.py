from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import check_ci_workflow as check  # noqa: E402 (import after sys.path setup)

GOOD = """\
name: CI

on:
  pull_request:
  push:
    branches: [main, merge-train/current]
  workflow_dispatch:

permissions:
  contents: read

jobs:
  a:
    permissions:
      # Only what checkout needs.
      contents: read
    runs-on: ubuntu-24.04
    steps:
      - run: echo "${{ github.ref }}"
"""


class Rights(unittest.TestCase):
    def test_contents_read_everywhere_passes(self) -> None:
        self.assertEqual(check.violations(GOOD), [])

    def test_secret_references_fail(self) -> None:
        for line in [
            "      - run: echo ${{ secrets.TOKEN }}",
            "      - run: echo ${{ secrets['TOKEN'] }}",
            "    secrets: inherit",
            "        env: {T: '${{ github.token }}'}",
            "        with: {token: '${{ github . token }}'}",
        ]:
            self.assertEqual(len(check.violations(GOOD + line + "\n")), 1, line)

    def test_messages_name_the_line(self) -> None:
        lines = GOOD.count("\n")
        text = GOOD + "      - run: echo ${{ secrets.TOKEN }}\n"
        self.assertEqual(
            check.violations(text), [f"ci.yml:{lines + 1}: references a secret or the job token"]
        )
        # The job-level block is on line 14 of GOOD.
        text = GOOD.replace("      contents: read\n    runs-on", "      contents: write\n    runs-on")
        self.assertEqual(
            check.violations(text), ["ci.yml:14: permissions must be exactly `contents: read`"]
        )

    def test_wider_permissions_fail(self) -> None:
        for old, new in [
            ("  contents: read\n\njobs", "  contents: write\n\njobs"),
            ("  contents: read\n\njobs", "  contents: read\n  pull-requests: write\n\njobs"),
            ("permissions:\n  contents: read\n\n", "permissions: write-all\n\n"),
            ("permissions:\n  contents: read\n\n", "permissions: {contents: read}\n\n"),
            (
                "    permissions:\n      # Only what checkout needs.\n      contents: read\n",
                "    permissions: read-all\n",
            ),
            ("      contents: read\n    runs-on", "      contents: read\n      id-token: write\n    runs-on"),
            (
                "    permissions:\n      # Only what checkout needs.\n      contents: read\n",
                "    permissions:\n",
            ),
        ]:
            self.assertIn(old, GOOD)
            self.assertNotEqual(check.violations(GOOD.replace(old, new)), [], new)

    def test_missing_workflow_permissions_fail(self) -> None:
        # Without a workflow-level block the token gets the repository's default rights.
        text = GOOD.replace("permissions:\n  contents: read\n\njobs", "jobs")
        self.assertEqual(len(check.violations(text)), 1)

    def test_repository_ci_yml_passes(self) -> None:
        self.assertEqual(check.violations(check.CI_YML.read_text(encoding="utf-8")), [])


class Trigger(unittest.TestCase):
    def test_push_branches_are_exactly_main_and_the_two_trains(self) -> None:
        # SPEC-0027 section 9, decision 0147: merge-train/revert-* and merge-train/probe get no
        # push CI (a revert PR gets pull_request CI), and merge-lock triggers nothing.
        branches = check.push_branches(check.CI_YML.read_text(encoding="utf-8"))
        self.assertEqual(branches, ["main", "merge-train/current", "merge-train/shadow"])
        # Literal names: a glob character would let other branches match.
        self.assertFalse(any(c in b for b in branches for c in "*?[]!+"))
        for other in ["merge-train/revert-x", "merge-train/probe", "merge-lock", "merge-train/current-2"]:
            self.assertNotIn(other, branches)

    def test_push_branches_parses_flow_and_block_lists(self) -> None:
        self.assertEqual(check.push_branches(GOOD), ["main", "merge-train/current"])
        block = GOOD.replace(
            "branches: [main, merge-train/current]", "branches:\n      - main\n      - 'x/y'"
        )
        self.assertEqual(check.push_branches(block), ["main", "x/y"])
        self.assertIsNone(check.push_branches(GOOD.replace("  push:\n", "  pushed:\n")))


if __name__ == "__main__":
    unittest.main()
