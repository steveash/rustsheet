from __future__ import annotations

import subprocess
import sys
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


class DocsReaders(unittest.TestCase):
    def test_literal_naming_skipped_docs_path_fails(self) -> None:
        for line in [
            'let s = include_str!("../../../docs/tasks/README.md");',
            "root = Path(ROOT) / 'docs'",
            'fs::read(".claude/skills/x.md")',
            'let p = root.join("docs/status.md");',
        ]:
            self.assertEqual(len(readers.violations("crates/x/src/a.rs", line)), 1, line)

    def test_code_read_paths_and_prose_pass(self) -> None:
        for line in [
            'const SEMANTICS_DIR: &str = "docs/excel-semantics/";',
            'root.join("docs/excel-semantics")',
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
