"""Tests for merge_renumber.py (`make renumber`) and the carry rule's renumber case
(id_renumber.py, decision 0224), over a temporary origin."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import id_renumber  # noqa: E402 (the sys.path line above must run first)
import merge_ready  # noqa: E402
import merge_renumber  # noqa: E402
from merge_catchup import Refused  # noqa: E402
from test_merge_catchup import TempOrigin, run  # noqa: E402

BASELINE = "eval/baseline/probes.txt"
PR_TASK = "docs/tasks/M2/T-0101-pr-task.md"
NEW_TASK = "docs/tasks/M2/T-0150-pr-task.md"
PR_RECORD = "docs/decisions/0002-pr-rule.md"
NEW_RECORD = "docs/decisions/0009-pr-rule.md"
FRESH = {"T-0101": "T-0150", "0002": "0009"}


def task(tid: str, title: str) -> str:
    return f"---\nid: {tid}\ntitle: {title}\nmilestone: M2\n---\n\n## Goal\n{title}; decision 0002.\n"


class Substitute(unittest.TestCase):
    def test_whole_ids_only(self) -> None:
        mapping = {"T-0101": "T-0150", "0002": "0009"}
        text = "T-0101, T-01012, XT-0101, T-0101-slug, 0002, 0.0002, 10002, T-0002, 0002-rule.md, 0002."
        self.assertEqual(
            id_renumber.substitute(text, mapping),
            "T-0150, T-01012, XT-0101, T-0150-slug, 0009, 0.0002, 10002, T-0002, 0009-rule.md, 0009.",
        )

    def test_no_chains(self) -> None:
        self.assertEqual(
            id_renumber.substitute("T-0001 T-0002", {"T-0001": "T-0002", "T-0002": "T-0003"}), "T-0002 T-0003"
        )

    def test_id_path(self) -> None:
        self.assertEqual(id_renumber.id_path(PR_TASK), ("docs/tasks/M2/", "T-0101", "-pr-task.md"))
        self.assertEqual(id_renumber.id_path(PR_RECORD), ("docs/decisions/", "0002", "-pr-rule.md"))
        self.assertIsNone(id_renumber.id_path("docs/decisions/README.md"))
        self.assertIsNone(id_renumber.id_path("scripts/T-0101-x.md"))


class Renumber(TempOrigin):
    """`main` and the PR branch `agent/x` both add task T-0101 and decision 0002 after a common
    commit, and both edit notes.md, in lines far apart."""

    def setUp(self) -> None:
        self.make_origin()
        self.write(BASELINE, "F\tprobes/a\tA1\n")
        self.write("eval/suites/probes/suite.toml", 'name = "probes"\ngate = true\n')
        self.write("docs/tasks/M2/T-0100-base.md", task("T-0100", "base"))
        self.write("docs/decisions/0001-base.md", "# 0001 — base\n")
        self.write("notes.md", "top\n" + "".join(f"filler {n}\n" for n in range(8)) + "bottom\n")
        self.base = self.commit("base")
        self.git("push", "-q", "origin", "main")
        self.git("checkout", "-q", "-b", "agent/x")
        self.write(PR_TASK, task("T-0101", "pr task"))
        self.write(PR_RECORD, "# 0002 — pr rule\n\nSee T-0101.\n")
        self.edit("notes.md", "bottom\n", "bottom\nPR: T-0101 and decision 0002.\n")
        self.before = self.commit("pr")
        self.git("push", "-q", "origin", "agent/x")
        self.git("checkout", "-q", "main")
        self.write("docs/tasks/M2/T-0101-main-task.md", task("T-0101", "main task"))
        self.write("docs/decisions/0002-main-rule.md", "# 0002 — main rule\n")
        self.edit("notes.md", "top\n", "top\nmain: T-0101 and decision 0002 are main's.\n")
        self.main = self.commit("main moves")
        self.git("push", "-q", "origin", "main")
        self.allocated: list[tuple[str, str]] = []

    def edit(self, path: str, old: str, new: str) -> None:
        self.write(path, (self.clone / path).read_text().replace(old, new, 1))

    def allocate(self, old: str, milestone: str, taken: set[str]) -> str:
        self.allocated.append((old, milestone))
        self.assertNotIn(FRESH[old], taken)
        return FRESH[old]

    @staticmethod
    def no_record(root: Path, new_cases: Path) -> int:
        raise AssertionError("no list conflicted, so nothing is re-recorded")

    def show(self, commit: str, path: str) -> str:
        return run("git", "show", f"{commit}:{path}", cwd=self.origin)

    def renumbered(self) -> str:
        return merge_renumber.renumber(self.clone, "agent/x", self.allocate, self.no_record)

    def test_renumbers_the_prs_files_and_lines_and_the_carry_accepts_it(self) -> None:
        start = self.git("rev-parse", "HEAD")
        head = self.renumbered()
        self.assertEqual(self.git("rev-parse", "HEAD"), start)  # the checkout is put back
        self.assertEqual(run("git", "rev-parse", "agent/x", cwd=self.origin), head)
        self.assertEqual(
            run("git", "rev-parse", f"{head}^1", f"{head}^2", cwd=self.origin).split(),
            [self.before, self.main],
        )
        self.assertEqual(self.allocated, [("0002", ""), ("T-0101", "M2")])
        files = run("git", "ls-tree", "-r", "--name-only", head, "docs", cwd=self.origin).split()
        self.assertIn(NEW_TASK, files)
        self.assertIn(NEW_RECORD, files)
        self.assertNotIn(PR_TASK, files)
        self.assertNotIn(PR_RECORD, files)
        self.assertEqual(self.show(head, NEW_TASK), task("T-0150", "pr task").replace("0002", "0009").strip())
        self.assertEqual(self.show(head, NEW_RECORD), "# 0009 — pr rule\n\nSee T-0150.")
        # main's own files and main's line keep the old ids; the PR's line gets the new ones.
        self.assertEqual(
            self.show(head, "docs/tasks/M2/T-0101-main-task.md"), task("T-0101", "main task").strip()
        )
        notes = self.show(head, "notes.md")
        self.assertIn("main: T-0101 and decision 0002 are main's.", notes)
        self.assertIn("PR: T-0150 and decision 0009.", notes)
        self.assertIsNone(merge_ready.Repo(self.clone).carry_problem(self.before, head))

    def manual(self, change: dict[str, str | None]) -> str:
        """A merge of main into the PR head with `change` (path -> text, None to delete) made by
        hand on top, committed and pushed."""
        self.git("checkout", "-q", "--detach", self.before)
        self.git("merge", "-q", "--no-ff", "--no-commit", "main")
        for path, text in change.items():
            if text is None:
                self.git("rm", "-q", path)
            else:
                self.write(path, text)
        head = self.commit("hand merge")
        self.git("push", "-q", "origin", f"{head}:refs/heads/agent/x")
        self.git("checkout", "-q", "main")
        return head

    def by_hand(self, **overrides: str) -> dict[str, str | None]:
        """What `make renumber` writes, with `overrides` (keyed by an alias) on top."""
        notes = self.git("show", "main:notes.md") + "\n"
        out: dict[str, str | None] = {
            PR_TASK: None,
            PR_RECORD: None,
            NEW_TASK: task("T-0150", "pr task").replace("0002", "0009"),
            NEW_RECORD: "# 0009 — pr rule\n\nSee T-0150.\n",
            "notes.md": notes.replace("bottom\n", "bottom\nPR: T-0150 and decision 0009.\n"),
        }
        aliases = {"task": NEW_TASK, "record": NEW_RECORD, "notes": "notes.md"}
        out.update({aliases[k]: v for k, v in overrides.items()})
        return out

    def carry(self, head: str) -> str | None:
        repo = merge_ready.Repo(self.clone)
        repo.fetch_base()
        return repo.carry_problem(self.before, head)

    def test_the_same_renumber_by_hand_is_accepted(self) -> None:
        self.git("checkout", "-q", "main")
        change = self.by_hand()
        self.assertIsNone(self.carry(self.manual(change)))

    def test_a_substitution_in_a_line_main_owns_is_refused(self) -> None:
        change = self.by_hand()
        notes = str(change["notes.md"]).replace(
            "main: T-0101 and decision 0002", "main: T-0150 and decision 0009"
        )
        problem = self.carry(self.manual(self.by_hand(notes=notes)))
        self.assertIsNotNone(problem)
        self.assertIn("notes.md differs", str(problem))

    def test_another_edit_in_a_renamed_file_is_refused(self) -> None:
        problem = self.carry(self.manual(self.by_hand(record="# 0009 — pr rule, edited\n\nSee T-0150.\n")))
        self.assertIn(NEW_RECORD, str(problem))

    def test_a_rename_to_an_id_main_has_is_refused(self) -> None:
        change = self.by_hand()
        del change[NEW_TASK]
        change["docs/tasks/M2/T-0100-pr-task.md"] = task("T-0100", "pr task").replace("0002", "0009")
        change["notes.md"] = str(change["notes.md"]).replace("PR: T-0150", "PR: T-0100")
        problem = self.carry(self.manual(change))
        self.assertIn("not a fresh id", str(problem))

    def test_a_rename_of_a_file_main_owns_is_refused(self) -> None:
        change: dict[str, str | None] = {
            "docs/decisions/0001-base.md": None,
            "docs/decisions/0009-base.md": "# 0009 — base\n",
        }
        self.assertIsNotNone(self.carry(self.manual(change)))

    def test_no_collision_is_refused_and_nothing_pushed(self) -> None:
        self.git("checkout", "-q", "main")
        self.git("rm", "-q", "docs/tasks/M2/T-0101-main-task.md", "docs/decisions/0002-main-rule.md")
        self.commit("main drops its copies")
        self.git("push", "-q", "origin", "main")
        with self.assertRaisesRegex(Refused, "adds no task or decision file"):
            self.renumbered()
        self.assertEqual(run("git", "rev-parse", "agent/x", cwd=self.origin), self.before)
        self.assertEqual(self.allocated, [])

    def test_a_conflict_outside_the_lists_is_refused_before_any_id_is_reserved(self) -> None:
        self.git("checkout", "-q", "main")
        self.edit("notes.md", "bottom\n", "bottom\nmain's own last line\n")
        self.commit("main conflicts")
        self.git("push", "-q", "origin", "main")
        with self.assertRaisesRegex(Refused, "conflicts outside the eval-gated baseline lists"):
            self.renumbered()
        self.assertEqual(self.allocated, [])
        self.assertEqual(run("git", "rev-parse", "agent/x", cwd=self.origin), self.before)

    def test_a_registry_refusal_pushes_nothing(self) -> None:
        def refuse(old: str, milestone: str, taken: set[str]) -> str:
            raise Refused("the id registry could not reserve a new id")

        with self.assertRaisesRegex(Refused, "id registry"):
            merge_renumber.renumber(self.clone, "agent/x", refuse, self.no_record)
        self.assertEqual(run("git", "rev-parse", "agent/x", cwd=self.origin), self.before)
        self.assertEqual(self.git("status", "--porcelain", "--untracked-files=no"), "")

    def test_a_list_conflict_is_resolved_and_re_recorded(self) -> None:
        self.git("checkout", "-q", "agent/x")
        self.write(BASELINE, "F\tprobes/pr\tP1\n")
        self.before = self.commit("pr list")
        self.git("push", "-q", "origin", "agent/x")
        self.git("checkout", "-q", "main")
        self.write(BASELINE, "F\tprobes/main\tM1\n")
        self.commit("main list")
        self.git("push", "-q", "origin", "main")
        recorded: list[Path] = []

        def record(root: Path, new_cases: Path) -> int:
            recorded.append(root)
            (root / BASELINE).write_text("F\tprobes/main\tM1\n")
            return 0

        head = merge_renumber.renumber(self.clone, "agent/x", self.allocate, record)
        self.assertEqual(recorded, [self.clone])
        self.assertIsNone(merge_ready.Repo(self.clone).carry_problem(self.before, head))

    def test_milestone_of(self) -> None:
        self.assertEqual(merge_renumber.milestone_of(self.clone, self.before, PR_TASK), "M2")
        self.assertEqual(merge_renumber.milestone_of(self.clone, self.before, PR_RECORD), "")


if __name__ == "__main__":
    unittest.main()
