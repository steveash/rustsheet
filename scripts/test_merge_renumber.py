"""Tests for merge_renumber.py (`make renumber`) and the carry rule's renumber case
(id_renumber.py, decision 0224), over a temporary origin."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import id_registry  # noqa: E402 (the sys.path line above must run first)
import id_renumber  # noqa: E402
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
        self.write("docs/notes.md", "top\n" + "".join(f"filler {n}\n" for n in range(8)) + "bottom\n")
        self.base = self.commit("base")
        self.git("push", "-q", "origin", "main")
        self.git("checkout", "-q", "-b", "agent/x")
        self.write(PR_TASK, task("T-0101", "pr task"))
        self.write(PR_RECORD, "# 0002 — pr rule\n\nSee T-0101.\n")
        self.edit("docs/notes.md", "bottom\n", "bottom\nPR: T-0101 and decision 0002.\n")
        self.before = self.commit("pr")
        self.git("push", "-q", "origin", "agent/x")
        self.git("checkout", "-q", "main")
        self.write("docs/tasks/M2/T-0101-main-task.md", task("T-0101", "main task"))
        self.write("docs/decisions/0002-main-rule.md", "# 0002 — main rule\n")
        self.edit("docs/notes.md", "top\n", "top\nmain: T-0101 and decision 0002 are main's.\n")
        self.main = self.commit("main moves")
        self.git("push", "-q", "origin", "main")
        self.allocated: list[tuple[str, str]] = []
        self.claim(*FRESH.values())  # what the fake allocator hands out, as the registry claimed it

    def claim(self, *ids: str) -> None:
        """Pushes the id registry with `ids` claimed (decision 0215), as `id_registry.claim` does."""
        registry = id_registry.commit(list(ids), self.clone)
        self.git("push", "-q", "-f", "origin", f"{registry}:{id_registry.REF}")

    def edit(self, path: str, old: str, new: str) -> None:
        self.write(path, (self.clone / path).read_text().replace(old, new, 1))

    def allocate(self, root: Path, old: str, milestone: str, taken: set[str]) -> str:
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
        notes = self.show(head, "docs/notes.md")
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
        self.git("push", "-q", "-f", "origin", f"{head}:refs/heads/agent/x")
        self.git("checkout", "-q", "main")
        return head

    def by_hand(self, **overrides: str) -> dict[str, str | None]:
        """What `make renumber` writes, with `overrides` (keyed by an alias) on top."""
        notes = self.git("show", "main:docs/notes.md") + "\n"
        out: dict[str, str | None] = {
            PR_TASK: None,
            PR_RECORD: None,
            NEW_TASK: task("T-0150", "pr task").replace("0002", "0009"),
            NEW_RECORD: "# 0009 — pr rule\n\nSee T-0150.\n",
            "docs/notes.md": notes.replace("bottom\n", "bottom\nPR: T-0150 and decision 0009.\n"),
        }
        aliases = {"task": NEW_TASK, "record": NEW_RECORD, "notes": "docs/notes.md"}
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
        notes = str(change["docs/notes.md"]).replace(
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
        change["docs/notes.md"] = str(change["docs/notes.md"]).replace("PR: T-0150", "PR: T-0100")
        problem = self.carry(self.manual(change))
        self.assertIn("not a fresh id", str(problem))

    def test_a_new_id_not_claimed_in_the_registry_is_refused(self) -> None:
        self.claim("T-0150")  # 0009 is free but nobody allocated it
        problem = self.carry(self.manual(self.by_hand()))
        self.assertIn("0009 is not claimed in the id registry", str(problem))

    def test_an_unreadable_registry_refuses(self) -> None:
        head = self.manual(self.by_hand())
        repo = merge_ready.Repo(self.clone)
        repo.fetch_base()
        self.git("remote", "set-url", "origin", str(self.origin) + "-gone")
        self.assertIn("the id registry cannot be read", str(repo.carry_problem(self.before, head)))
        self.git("remote", "set-url", "origin", str(self.origin))
        self.git("push", "-q", "origin", f":{id_registry.REF}")  # an absent registry claims nothing
        self.assertIn("not claimed in the id registry", str(self.carry(head)))

    def test_a_rename_without_a_real_collision_is_refused(self) -> None:
        # The PR adds decision 0003, which main does not have; a merge moves it to a claimed id.
        self.git("checkout", "-q", "agent/x")
        self.write("docs/decisions/0003-new.md", "# 0003 — new\n")
        self.before = self.commit("pr adds 0003")
        self.git("push", "-q", "origin", "agent/x")
        self.git("checkout", "-q", "main")
        self.claim(*FRESH.values(), "0010")
        change = self.by_hand()
        change["docs/decisions/0003-new.md"] = None
        change["docs/decisions/0010-new.md"] = "# 0010 — new\n"
        problem = self.carry(self.manual(change))
        self.assertIn("0003 does not collide with a file on main", str(problem))

    def test_an_id_in_code_the_pr_added_refuses_the_renumber_and_the_carry(self) -> None:
        # Security review of #2699: `MAX_DEPTH = 0002` is code, not a reference; nobody renumbers it.
        self.git("checkout", "-q", "agent/x")
        self.write("src/lim.rs", 'const MAX_DEPTH: usize = 0002;\nconst NAME: &str = "0002";\n')
        self.before = self.commit("pr adds code")
        self.git("push", "-q", "origin", "agent/x")
        self.git("checkout", "-q", "main")
        with self.assertRaisesRegex(Refused, "outside docs/ name a colliding id \\(src/lim.rs\\)"):
            self.renumbered()
        self.assertEqual(self.allocated, [])  # refused before any id was reserved
        self.assertEqual(run("git", "rev-parse", "agent/x", cwd=self.origin), self.before)
        for code in ('const MAX_DEPTH: usize = 0009;\nconst NAME: &str = "0009";\n', None):
            change = self.by_hand()
            if code is not None:
                change["src/lim.rs"] = code
            with self.subTest(code=code):
                problem = self.carry(self.manual(change))
                self.assertIn("outside docs/ name an old id (src/lim.rs)", str(problem))

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
        self.edit("docs/notes.md", "bottom\n", "bottom\nmain's own last line\n")
        self.commit("main conflicts")
        self.git("push", "-q", "origin", "main")
        with self.assertRaisesRegex(Refused, "conflicts outside the eval-gated baseline lists"):
            self.renumbered()
        self.assertEqual(self.allocated, [])
        self.assertEqual(run("git", "rev-parse", "agent/x", cwd=self.origin), self.before)

    def test_a_registry_refusal_pushes_nothing(self) -> None:
        def refuse(root: Path, old: str, milestone: str, taken: set[str]) -> str:
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

    def test_the_real_allocator_reserves_only_for_its_own_checkout(self) -> None:
        # Its rules read this checkout's tasks and branches, so another root is refused, not misread.
        with self.assertRaisesRegex(Refused, "ids are reserved from the checkout"):
            merge_renumber.reserve(self.clone, "T-0101", "M2", set())


if __name__ == "__main__":
    unittest.main()
