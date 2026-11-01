"""The progress page (SPEC-0022, T-0070): the status block, judgment-block preservation and its
loud failures, the as-of stale line, the method block and byte-identical reruns."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import pytest
from test_progress_report import GIT, data, report, write_inputs

from rustsheet_tools import progress_report
from rustsheet_tools.progress_report import blocks, page
from rustsheet_tools.progress_report.markdown import md_text
from rustsheet_tools.progress_report.model import ProgressReportError, Run, run_of

RUN = Run("2026-10-19", "abc12345", ("s",), ())
DATA = data()
AS_OF = "<!-- as-of: 2026-10-19 abc12345 -->"
SECTIONS = {
    "goals": "## Goals",
    "campaign": "## Campaign stages",
    "since-last-run": "## Since the last run",
    "trends": "## Trends",
    "arguments": "## Argument coverage",
    "codebase": "## Codebase health",
}


def render(existing: str | None) -> str:
    """The page for RUN at its own commit, keeping the judgment text of `existing`."""
    return page.render(RUN, DATA, existing, SECTIONS)


def judgment_page(texts: dict[str, str]) -> str:
    """An existing page holding judgment block `name` with text `texts[name]` for each name."""
    parts = [
        blocks.open_marker("judgment", n) + t + blocks.close_marker("judgment") for n, t in texts.items()
    ]
    return "# old page\n\n" + "\n\n".join(parts) + "\n"


def run_main(tmp_path: Path, r: dict[str, Any] | None = None) -> Path:
    out = tmp_path / "pages"
    progress_report.main([*write_inputs(tmp_path, r or report()), "--out-dir", str(out)], GIT)
    return out


def test_a_report_without_the_run_record_fails_loudly() -> None:
    r = report()
    del r["run"]
    with pytest.raises(ProgressReportError, match="T-0080"):
        run_of(r)


def test_status_block_names_the_run_suites_and_links() -> None:
    r = report()
    r["run"]["skipped_suites"] = [{"name": "poi", "reason": "not fetched"}]
    run = run_of(r)
    assert (run.date, run.commit, run.suites) == ("2026-10-19", "abc12345", ("s",))
    assert [(s.name, s.reason) for s in run.skipped] == [("poi", "not fetched")]
    text = page.status(run)
    assert "The eval run of **2026-10-19** at commit `abc12345`" in text
    assert "- **Suites run (1):** s." in text
    assert "- **Suites skipped (1):** poi (not fetched)." in text
    assert "[docs/status.md](status.md)" in text
    assert "[docs/progress/functions.md](progress/functions.md)" in text
    unknown = page.status(Run("2026-10-19", None, (), ()))
    assert "an unknown commit (git was unavailable)" in unknown
    assert "**Suites run (0):** none." in unknown
    assert "**Suites skipped (0):** none." in unknown


def test_hostile_suite_names_are_escaped_in_the_status_block() -> None:
    text = page.status(Run("2026-10-19", "abc12345", ("[x](https://e.com)<b>",), ()))
    assert "[x](" not in text
    assert "<b>" not in text
    assert md_text("a_*b*_ `c` |") == "a\\_\\*b\\*\\_ &#96;c&#96; \\|"


def test_a_first_run_writes_every_block_in_order_with_placeholders() -> None:
    text = render(None)
    names = [m.group(3) for m in blocks.MARKER.finditer(text) if not m.group(1)]
    assert names == [
        "status",
        "bottom-line",
        "goals",
        "campaign",
        "since-last-run",
        "trends",
        "parity",
        "top-functions",
        "evidence",
        "arguments",
        "codebase",
        "readiness",
        "risks",
        "method",
    ]
    parsed = page.parse(text)
    assert all(parsed[n].text == blocks.PLACEHOLDER for n in page.JUDGMENT_BLOCKS)
    assert "not yet updated" not in text
    assert "## Method" in parsed["method"].text
    assert text.endswith("<!-- /progress:generated -->\n")


def test_judgment_text_is_kept_byte_for_byte() -> None:
    texts = {
        "bottom-line": f"\n{AS_OF}\nHeadline: 5 of 6.\n\n- one  \n- two\t\n",
        "readiness": f"{AS_OF}\r\n*Likely*: read and compute.\r\né≤ no newline at the end",
        "risks": f"\n\n  {AS_OF}\n1. The risk (T-0071).\n   progress: plain prose is fine\n\n\n",
    }
    new = render(judgment_page(texts))
    parsed = page.parse(new)
    for name, text in texts.items():
        assert parsed[name].text == text
        assert blocks.open_marker("judgment", name) + text + blocks.close_marker("judgment") in new
    assert "# old page" not in new
    assert "not yet updated" not in new


def test_judgment_text_survives_main_with_crlf_on_disk(tmp_path: Path) -> None:
    out = tmp_path / "pages"
    out.mkdir()
    text = f"{AS_OF}\r\nKept as is.\r\n"
    (out / "progress.md").write_bytes(judgment_page({"risks": text}).encode())
    run_main(tmp_path)
    raw = (out / "progress.md").read_bytes()
    assert ("<!-- progress:judgment risks -->" + text + "<!-- /progress:judgment -->").encode() in raw


def test_a_missing_judgment_block_gets_the_placeholder() -> None:
    new = render(judgment_page({"risks": f"\n{AS_OF}\nOnly risks.\n"}))
    parsed = page.parse(new)
    assert parsed["bottom-line"].text == blocks.PLACEHOLDER
    assert parsed["readiness"].text == blocks.PLACEHOLDER
    assert parsed["risks"].text == f"\n{AS_OF}\nOnly risks.\n"


J, G = "judgment", "generated"


@pytest.mark.parametrize(
    ("existing", "error"),
    [
        (judgment_page({"summary": "x"}), "unknown judgment block name 'summary'"),
        (blocks.open_marker(G, "summary") + blocks.close_marker(G), "unknown generated block name 'summary'"),
        (
            blocks.open_marker(G, "status") + blocks.open_marker(G, "method") + blocks.close_marker(G) * 2,
            "nested marker inside block 'status'",
        ),
        (
            blocks.open_marker(G, "status") + blocks.open_marker(J, "risks") + blocks.close_marker(J),
            "nested marker inside block 'status'",
        ),
        (judgment_page({"risks": blocks.open_marker(J, "readiness")}), "judgment block 'risks'.*contains"),
        (judgment_page({"risks": blocks.close_marker(G)}), "judgment block 'risks'.*contains"),
        (judgment_page({"risks": "<!-- progress:note -->"}), "malformed progress marker"),
        (judgment_page({"risks": "<!--progress:judgment risks-->"}), "malformed progress marker"),
        (judgment_page({"risks": "a <!-- note"}), "malformed progress marker"),
        (judgment_page({"risks": "a"}) + "<!-- note\n", "line 4: unterminated HTML comment"),
        (blocks.open_marker(J, "risks") + "\ntext\n", "block 'risks' is never closed"),
        ("text\n" + blocks.close_marker(J), "closes a judgment block that is not open"),
        (
            blocks.open_marker(G, "status") + blocks.close_marker(J),
            "unbalanced marker: closes a judgment block, but the open block 'status'",
        ),
        ("<!-- progress:judgment -->", "malformed progress marker"),
        ("<!-- /progress:judgment risks -->", "malformed progress marker"),
        (judgment_page({"risks": "a"}) + judgment_page({"risks": "b"}), "block 'risks' appears twice"),
    ],
)
def test_bad_markers_fail_loudly(tmp_path: Path, existing: str, error: str) -> None:
    with pytest.raises(ProgressReportError, match=error):
        render(existing)
    out = tmp_path / "pages"
    out.mkdir()
    (out / "progress.md").write_text(existing)
    with pytest.raises(SystemExit):
        run_main(tmp_path)
    assert (out / "progress.md").read_text() == existing
    assert not (out / "parity.md").exists()


@pytest.mark.parametrize(
    ("as_of", "line"),
    [
        (AS_OF, None),
        (
            "<!-- as-of: 2026-10-18 abc12345 -->",
            "_Written for the run of 2026-10-18; not yet updated for this run._",
        ),
        (
            "<!-- as-of: 2026-10-19 0123abcd -->",
            "_Written for the run of 2026-10-19; not yet updated for this run._",
        ),
        ("as of yesterday", "_Written for an unnamed run (no as-of line); not yet updated for this run._"),
    ],
)
def test_the_stale_line_when_the_as_of_does_not_name_the_run(as_of: str, line: str | None) -> None:
    text = f"\n{as_of}\nThe text.\n"
    new = render(judgment_page({"readiness": text}))
    section = new.split("## Real-world readiness\n\n", 1)[1].split("## Risks", 1)[0]
    if line is None:
        assert section.startswith(blocks.open_marker("judgment", "readiness"))
        assert "not yet updated" not in new
    else:
        assert section.startswith(f"{line}\n\n{blocks.open_marker('judgment', 'readiness')}")
        assert new.count("not yet updated") == 1
    # The stale text is still shown, unchanged.
    assert page.parse(new)["readiness"].text == text


def test_a_run_without_a_commit_never_matches_an_as_of() -> None:
    assert blocks.stale_line(f"\n{AS_OF}\n", Run("2026-10-19", None, (), ())) is not None
    assert blocks.stale_line(None, RUN) is None
    assert blocks.stale_line(blocks.PLACEHOLDER, RUN) is None


def test_two_runs_on_the_same_inputs_write_byte_identical_files(tmp_path: Path) -> None:
    def snapshot(out: Path) -> dict[str, bytes]:
        return {str(p.relative_to(out)): p.read_bytes() for p in sorted(out.rglob("*")) if p.is_file()}

    out = run_main(tmp_path)
    first = snapshot(out)
    assert "progress.md" in first
    run_main(tmp_path)
    assert snapshot(out) == first
    # With judgment text written by the steward, the same holds.
    page_text = (out / "progress.md").read_text()
    written = page_text.replace(blocks.PLACEHOLDER, f"\n{AS_OF}\nWritten.\n", 1)
    (out / "progress.md").write_text(written)
    run_main(tmp_path)
    assert (out / "progress.md").read_text() == written
    second = snapshot(out)
    run_main(tmp_path)
    assert snapshot(out) == second
