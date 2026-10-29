from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_governance as gov  # noqa: E402 (import after sys.path setup)
import merge_flip  # noqa: E402
from gh_client import FakeGh, Reply  # noqa: E402

SWITCH = merge_flip.SWITCH
TRAIN = '# The switch.\nmode = "train"\n'
FLIP = '# The switch.\nmode = "lock"\n'


def run(root: Path, *args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(root), *args], capture_output=True, text=True, check=True
    ).stdout.strip()


class Repo:
    """A scratch repository whose `main` holds the switch at `mode = "train"`, plus a branch."""

    def __init__(self, tmp: str) -> None:
        self.root = Path(tmp) / "work"
        self.root.mkdir()
        run(self.root, "init", "-q", "-b", "main")
        run(self.root, "config", "user.name", "t")
        run(self.root, "config", "user.email", "t@invalid")
        self.write(SWITCH, TRAIN)
        self.write("docs/a.md", "a\n")
        self.commit("base")
        self.base = run(self.root, "rev-parse", "HEAD")
        run(self.root, "checkout", "-q", "-b", "flip")

    def write(self, path: str, text: str) -> None:
        file = self.root / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(text)

    def commit(self, message: str) -> str:
        run(self.root, "add", "-A")
        run(self.root, "commit", "-q", "--allow-empty", "-m", message)
        return run(self.root, "rev-parse", "HEAD")

    def problems(self) -> list[str]:
        return merge_flip.local_problems(self.root, self.base, "HEAD")


class LocalRule(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Repo(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_the_one_line_lock_change_is_a_flip(self) -> None:
        self.repo.write(SWITCH, FLIP)
        self.repo.commit("flip")
        self.assertEqual(self.repo.problems(), [])

    def test_refused_diffs(self) -> None:
        cases: dict[str, dict[str, str]] = {
            "extra file": {SWITCH: FLIP, "docs/a.md": "b\n"},
            "extra line": {SWITCH: FLIP + "batch_max = 9\n"},
            "train": {SWITCH: '# The switch.\nmode = "train"\nx = 1\n'},
            "shadow": {SWITCH: '# The switch.\nmode = "shadow"\n'},
            "duplicate key": {SWITCH: FLIP + 'mode = "lock"\n'},
            "nested mode": {SWITCH: TRAIN + '[stop]\nmode = "lock"\n'},
            "mode not a string": {SWITCH: "mode = ['lock']\n"},
            "no change": {},
        }
        for name, files in cases.items():
            with self.subTest(name):
                run(self.repo.root, "checkout", "-q", "-B", "flip", self.repo.base)
                for path, text in files.items():
                    self.repo.write(path, text)
                self.repo.commit(name)
                self.assertNotEqual(self.repo.problems(), [], name)

    def test_a_symlink_a_type_change_an_executable_bit_and_a_rename_are_refused(self) -> None:
        target = self.repo.root / SWITCH

        def symlink() -> None:
            target.unlink()  # a dangling link: the only change is the type change itself
            target.symlink_to("flip.toml")

        def directory() -> None:
            target.unlink()
            self.repo.write(f"{SWITCH}/x", FLIP)

        def executable() -> None:
            self.repo.write(SWITCH, FLIP)
            target.chmod(0o755)

        def rename() -> None:
            run(self.repo.root, "mv", SWITCH, "scripts/merge_train_old.toml")
            self.repo.write(SWITCH, FLIP)
            run(self.repo.root, "add", "-A")
            # Without --no-renames git could pair the old blob with a new path.

        for name, change in {
            "symlink": symlink,
            "type change": directory,
            "exec": executable,
            "rename": rename,
        }.items():
            with self.subTest(name):
                run(self.repo.root, "checkout", "-q", "-f", "-B", "flip", self.repo.base)
                run(self.repo.root, "clean", "-qfd")
                change()
                self.repo.commit(name)
                self.assertNotEqual(self.repo.problems(), [], name)

    def test_a_blob_that_is_not_utf8_is_refused_not_raised(self) -> None:
        (self.repo.root / SWITCH).write_bytes(b'mode = "lock"\n# \xff\n')
        self.repo.commit("bytes")
        self.assertNotEqual(self.repo.problems(), [])


class PullRequest(unittest.TestCase):
    """`check` against a bare origin that serves refs/pull/7/head, and a fake files API."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Repo(self.tmp.name)
        origin = Path(self.tmp.name) / "origin.git"
        run(Path(self.tmp.name), "init", "-q", "--bare", str(origin))
        run(self.repo.root, "remote", "add", "origin", str(origin))
        run(self.repo.root, "push", "-q", "origin", "main")
        self.origin = origin

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def open_pr(self, text: str) -> str:
        self.repo.write(SWITCH, text)
        head = self.repo.commit("flip")
        run(self.repo.root, "push", "-q", "-f", "origin", f"{head}:refs/pull/7/head")
        return head

    def fake(self, heads: list[str], files: list[object]) -> FakeGh:
        """A client whose successive reads of PR 7 report `heads` in turn."""
        fake = FakeGh("o", "r", {("GET", "repos/o/r/pulls/7/files?per_page=100&page=1"): Reply(files)})
        pulls = iter(heads)
        send = fake.send

        def answer(request: object) -> object:
            if getattr(request, "path", "") == "repos/o/r/pulls/7":
                fake.requests.append(request)  # type: ignore[arg-type]
                return {"state": "open", "base": {"ref": "main"}, "head": {"sha": next(pulls)}}
            return send(request)  # type: ignore[arg-type]

        fake.send = answer  # type: ignore[method-assign]
        return fake

    def blob(self, head: str) -> str:
        return run(self.repo.root, "rev-parse", f"{head}:{SWITCH}")

    def test_a_flip_pr_passes_and_names_its_head(self) -> None:
        head = self.open_pr(FLIP)
        files = [{"status": "modified", "filename": SWITCH, "sha": self.blob(head)}]
        checked, problems = merge_flip.pr_problems(self.fake([head, head], files), 7, self.repo.root)
        self.assertEqual((checked, problems), (head, []))

    def test_a_head_that_moved_after_the_check_is_refused(self) -> None:
        head = self.open_pr(FLIP)
        files = [{"status": "modified", "filename": SWITCH, "sha": self.blob(head)}]
        _, problems = merge_flip.pr_problems(self.fake([head, "f" * 40], files), 7, self.repo.root)
        self.assertIn("the head moved during the check", problems)

    def test_the_files_api_must_list_the_one_file_and_its_blob(self) -> None:
        head = self.open_pr(FLIP)
        right = {"status": "modified", "filename": SWITCH, "sha": self.blob(head)}
        for name, files in {
            "two files": [right, {"status": "added", "filename": "x", "sha": "0" * 40}],
            "renamed": [{**right, "status": "renamed"}],
            "another blob": [{**right, "sha": "0" * 40}],
        }.items():
            with self.subTest(name):
                _, problems = merge_flip.pr_problems(self.fake([head, head], files), 7, self.repo.root)
                self.assertNotEqual(problems, [], name)

    def test_a_pr_that_sets_train_is_refused(self) -> None:
        head = self.open_pr('# The switch.\nmode = "shadow"\n')
        files = [{"status": "modified", "filename": SWITCH, "sha": self.blob(head)}]
        _, problems = merge_flip.pr_problems(self.fake([head, head], files), 7, self.repo.root)
        self.assertNotEqual(problems, [])


class GovernanceWaiver(unittest.TestCase):
    """Decision 0157: a flip waives the decision-record rule and no other governance rule."""

    FLIP_DIFF = (f"M\t{SWITCH}",)

    def test_a_flip_needs_no_decision_record(self) -> None:
        self.assertNotEqual(gov.failures(list(self.FLIP_DIFF), lambda _p: [], flip=False), [])
        self.assertEqual(gov.failures(list(self.FLIP_DIFF), lambda _p: [], flip=True), [])

    def test_the_waiver_leaves_the_other_rules_on(self) -> None:
        changed = [*self.FLIP_DIFF, "D\tcrates/x/tests/a.rs", "A\teval/suites/probes/cases/a.case.json"]
        found = gov.failures(changed, lambda _p: ["M1"], flip=True)
        heads = [lines[0] for lines in found]
        self.assertEqual(len(found), 2, heads)
        self.assertTrue(any("test files deleted" in h for h in heads))
        self.assertTrue(any("probe cases" in h for h in heads))
        self.assertFalse(any("decision record in docs/decisions" in h for h in heads))


if __name__ == "__main__":
    unittest.main()
