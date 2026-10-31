from __future__ import annotations

import sys
import unittest
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_governance as gov  # noqa: E402 (import after sys.path setup)


class ProtectedPaths(unittest.TestCase):
    def test_frozen_milestone_lists_are_protected(self) -> None:
        # Decision 0043: a line leaves eval/milestones/m1-probes.txt only by a decision record.
        self.assertTrue(gov.is_protected("eval/milestones/m1-probes.txt"))

    def test_rule_paths_and_deviations_files_are_protected(self) -> None:
        for path in [
            "CLAUDE.md",
            "Makefile",
            "scripts/tasks.py",
            ".claude/skills/steward/SKILL.md",
            "eval/suites/probes/deviations",
        ]:
            self.assertTrue(gov.is_protected(path), path)

    def test_ordinary_paths_are_not(self) -> None:
        for path in ["eval/suites/probes/cases/a.case.json", "crates/rustsheet-eval/src/main.rs", "eval/milestones"]:
            self.assertFalse(gov.is_protected(path), path)


class NewProbeCaseTags(unittest.TestCase):
    """Decision 0057: an added probes case has one milestone tag, M2 or later."""

    TAGS = {
        "eval/suites/probes/cases/m1.case.json": ["M1", "probe"],
        "eval/suites/probes/cases/untagged.case.json": ["probe", "lookup"],
        "eval/suites/probes/cases/two.case.json": ["M2", "M9"],
        "eval/suites/probes/cases/m2.case.json": ["M2", "probe"],
        "eval/suites/probes/cases/m9.case.json": ["probe", "M9"],
        "eval/suites/probes/cases/catalog/sum.case.json": ["M0"],
        "eval/suites/probes/cases/old.case.json": ["M1"],
    }

    def problems(self, *lines: str) -> list[str]:
        return gov.probe_tag_problems(lines, lambda path: self.TAGS[path])

    def test_added_m0_m1_untagged_or_twice_tagged_cases_fail(self) -> None:
        self.assertEqual(
            self.problems("A\teval/suites/probes/cases/m1.case.json"),
            ["eval/suites/probes/cases/m1.case.json (milestone tags: M1)"],
        )
        self.assertEqual(
            self.problems("A\teval/suites/probes/cases/untagged.case.json"),
            ["eval/suites/probes/cases/untagged.case.json (milestone tags: none)"],
        )
        self.assertEqual(
            self.problems("A\teval/suites/probes/cases/two.case.json"),
            ["eval/suites/probes/cases/two.case.json (milestone tags: M2, M9)"],
        )

    def test_added_m2_or_later_and_catalog_cases_pass(self) -> None:
        self.assertEqual(
            self.problems(
                "A\teval/suites/probes/cases/m2.case.json",
                "A\teval/suites/probes/cases/m9.case.json",
                "A\teval/suites/probes/cases/catalog/sum.case.json",
            ),
            [],
        )

    def test_modified_cases_and_other_files_are_not_checked(self) -> None:
        self.assertEqual(
            self.problems(
                "M\teval/suites/probes/cases/old.case.json",
                "A\teval/suites/probes/cases/m1.oracle.json",
                "A\teval/suites/smoke/cases/m1.case.json",
            ),
            [],
        )


class IssueTasks(unittest.TestCase):
    """Decision 0158: a diff with a task filed from a GitHub issue never changes a protected path."""

    ISSUE_TASKS = {"docs/tasks/M2/T-0500-from-issue.md"}

    def failures(self, *lines: str) -> list[list[str]]:
        return gov.failures(list(lines), lambda _p: ["M2"], flip=False, has_issue=lambda p: p in self.ISSUE_TASKS)

    def test_issue_task_with_a_protected_path_fails_even_with_a_decision_record(self) -> None:
        for status in ("A", "M"):
            found = self.failures(
                f"{status}\tdocs/tasks/M2/T-0500-from-issue.md",
                "M\t.claude/skills/work-loop/SKILL.md",
                "A\tdocs/decisions/0999-x.md",
            )
            self.assertEqual(len(found), 1, status)
            self.assertIn("GitHub issue", found[0][0])
            self.assertIn("  .claude/skills/work-loop/SKILL.md", found[0])

    def test_issue_task_with_code_baseline_and_docs_passes(self) -> None:
        self.assertEqual(
            self.failures(
                "A\tdocs/tasks/M2/T-0500-from-issue.md",
                "M\tcrates/rustsheet-engine/src/functions/text/mod.rs",
                "M\teval/baseline/probes.txt",
                "A\tdocs/research/issue-answer.md",
                "M\tdocs/excel-semantics/text.md",
            ),
            [],
        )

    def test_other_tasks_and_deleted_issue_tasks_keep_the_normal_rule(self) -> None:
        self.assertEqual(
            self.failures(
                "A\tdocs/tasks/M2/T-0501-planner.md",
                "D\tdocs/tasks/M2/T-0500-from-issue.md",
                "M\tscripts/tasks.py",
                "A\tdocs/decisions/0999-x.md",
            ),
            [],
        )

    def supply(self, *lines: str, added: list[tuple[str, str]] | None = None) -> list[list[str]]:
        return gov.failures(
            list(lines), lambda _p: ["M2"], flip=False, has_issue=lambda p: p in self.ISSUE_TASKS,
            added=lambda: added or [],
        )

    def test_issue_task_may_not_touch_a_manifest_lockfile_or_build_script(self) -> None:
        for path in ("crates/rustsheet-engine/Cargo.toml", "Cargo.lock", "tools/uv.lock",
                     "crates/rustsheet-engine/build.rs", "bindings/python/pyproject.toml", ".cargo/config.toml"):
            found = self.supply("A\tdocs/tasks/M2/T-0500-from-issue.md", f"M\t{path}")
            self.assertEqual(len(found), 1, path)
            self.assertIn("manifests", found[0][0])
            self.assertIn(f"  {path} (manifest, lockfile or build script)", found[0])

    def test_issue_task_may_not_add_a_lint_allow_or_unsafe(self) -> None:
        for text in ("#[allow(clippy::unwrap_used)]", "#![allow(dead_code)]", "    unsafe { f() }"):
            found = self.supply(
                "A\tdocs/tasks/M2/T-0500-from-issue.md", "M\tcrates/rustsheet-engine/src/lib.rs",
                added=[("crates/rustsheet-engine/src/lib.rs", text)],
            )
            self.assertEqual(len(found), 1, text)
            self.assertIn("lint allow or unsafe", found[0][-1])
        self.assertEqual(
            self.supply("A\tdocs/tasks/M2/T-0500-from-issue.md", "M\tcrates/a/src/lib.rs",
                        added=[("crates/a/src/lib.rs", "let safe = unsafely_named + 1;")]),
            [],
        )

    def test_ordinary_pr_may_touch_a_crate_manifest_and_add_allows(self) -> None:
        self.assertEqual(
            self.supply("A\tdocs/tasks/M2/T-0501-planner.md", "M\tcrates/rustsheet-engine/Cargo.toml",
                        "M\tCargo.lock", added=[("crates/a/src/lib.rs", "#[allow(dead_code)]")]),
            [],
        )

    def test_added_rust_lines_reads_plus_lines_of_rs_files_only(self) -> None:
        diff = (
            "diff --git a/crates/a/src/lib.rs b/crates/a/src/lib.rs\n--- a/crates/a/src/lib.rs\n"
            "+++ b/crates/a/src/lib.rs\n@@ -1,0 +2 @@\n+unsafe fn f() {}\n-old\n"
            "diff --git a/x.md b/x.md\n--- a/x.md\n+++ b/x.md\n@@ -0,0 +1 @@\n+unsafe prose\n"
            "diff --git a/crates/a/src/gone.rs b/crates/a/src/gone.rs\n--- a/crates/a/src/gone.rs\n+++ /dev/null\n"
        )
        self.assertEqual(gov.added_rust_lines(diff), [("crates/a/src/lib.rs", "unsafe fn f() {}")])

    def test_issue_task_files_reads_front_matter_through_has_issue(self) -> None:
        seen: list[str] = []

        def has_issue(path: str) -> bool:
            seen.append(path)
            return True

        self.assertEqual(
            gov.issue_task_files(["R100\tdocs/tasks/M0/T-0001-a.md\tdocs/tasks/M1/T-0001-a.md", "M\tdocs/tasks/README.md"],
                                 has_issue),
            ["docs/tasks/M1/T-0001-a.md"],
        )
        self.assertEqual(seen, ["docs/tasks/M1/T-0001-a.md"])


    TASK = "---\nid: T-0500\ntitle: \"x\"\nmilestone: M2\nlane: engine\nstatus: todo\n{extra}---\n"

    def real_guard(self, extra: str, *lines: str) -> list[list[str]]:
        """`failures` with the real `task_has_issue`, reading a task whose front matter adds `extra`."""
        with mock.patch.object(gov, "git", return_value=self.TASK.format(extra=extra)):
            return gov.failures(
                ["A\tdocs/tasks/M2/T-0500-from-issue.md", *lines], lambda _p: ["M2"], flip=False,
                added=lambda: [("crates/a/src/lib.rs", "#[allow(dead_code)]")],
            )

    def test_issue_origin_follow_up_gets_every_issue_guard(self) -> None:
        for extra in ("issue: 12\nissue_author: octo\n", "issue_origin: 12\n"):
            found = self.real_guard(extra, "M\t.claude/skills/x/SKILL.md", "A\tdocs/decisions/0999-x.md",
                                    "M\tCargo.lock", "M\tcrates/a/src/lib.rs")
            self.assertEqual(len(found), 2, extra)
            self.assertIn("protected paths", found[0][0])
            self.assertIn("  Cargo.lock (manifest, lockfile or build script)", found[1])
            self.assertIn("lint allow or unsafe", found[1][-1])

    def test_task_with_neither_key_keeps_the_normal_rule(self) -> None:
        self.assertEqual(
            self.real_guard("", "M\t.claude/skills/x/SKILL.md", "A\tdocs/decisions/0999-x.md",
                            "M\tCargo.lock", "M\tcrates/a/src/lib.rs"),
            [],
        )


if __name__ == "__main__":
    unittest.main()
