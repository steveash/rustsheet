"""`rustsheet-progress-report --check` (SPEC-0022 "Cited numbers" and "Plain text only", T-0074)."""

from __future__ import annotations

from pathlib import Path

import pytest
from test_progress_page import AS_OF, DATA, RUN, SECTIONS, judgment_page

from rustsheet_tools import progress_report
from rustsheet_tools.progress_report import blocks, check, page
from rustsheet_tools.progress_report.model import ProgressReportError

# A status block that also prints a percentage and a thousands-separated count to cite.
STATUS = (
    "The eval run of **2026-10-19** at commit `abc12345` (report.json `run`).\n\n"
    "- 97.5% of 1,234 cells; 12 functions."
)


def a_page(texts: dict[str, str] | None = None, status: str = STATUS) -> str:
    """A page whose judgment blocks hold `texts` (every other block: the as-of line alone)."""
    full = dict.fromkeys(page.JUDGMENT_BLOCKS, f"\n{AS_OF}\n") | (texts or {})
    return "\n\n".join(
        [
            blocks.generated("status", status),
            judgment_page(full),
            blocks.generated("method", page.METHOD),
        ]
    )


def offenders(text: str) -> list[str]:
    return check.offenders(a_page({"risks": f"\n{AS_OF}\n{text}\n"}))


def test_a_rendered_page_with_current_as_of_lines_passes() -> None:
    # The real generator's status block: keeps STATUS_RUN in step with page.status.
    texts = dict.fromkeys(page.JUDGMENT_BLOCKS, f"\n{AS_OF}\nSee the suites run (1) above.\n")
    rendered = page.render(RUN, DATA, judgment_page(texts), SECTIONS)
    assert check.offenders(rendered) == []


def test_cited_numbers_and_percentages_pass() -> None:
    assert offenders("- 97.5% of 1,234 cells, and 12 functions, are cited.") == []


@pytest.mark.parametrize(
    ("text", "number"),
    [
        ("13 functions are off.", "13"),
        ("97.6% of cells.", "97.6%"),
        ("97.5 of cells.", "97.5"),
        ("1234 cells.", "1234"),
    ],
)
def test_an_uncited_number_fails(text: str, number: str) -> None:
    assert offenders(text) == [f"risks: the number {number} appears in no generated block"]


@pytest.mark.parametrize(
    "text",
    [
        "Since 2026-10-01 it improved.",
        "T-0074 and SPEC-0022 own it.",
        "PR #1858 and decision 0118 decided it; Decision 0120 too.",
        "The M2 and M10 milestones.",
    ],
)
def test_ignored_forms_are_not_numbers(text: str) -> None:
    assert offenders(text) == []


def test_numbers_inside_the_as_of_comment_are_ignored() -> None:
    assert check.numbers(AS_OF) == []
    assert offenders("") == []


@pytest.mark.parametrize(
    ("as_of", "error"),
    [
        ("<!-- as-of: 2026-10-18 abc12345 -->", "risks: as-of 2026-10-18 abc12345 does not name"),
        ("<!-- as-of: 2026-10-19 def67890 -->", "risks: as-of 2026-10-19 def67890 does not name"),
        ("Plain text first.", "risks: no as-of line"),
    ],
)
def test_a_stale_or_missing_as_of_fails(as_of: str, error: str) -> None:
    found = check.offenders(a_page({"risks": f"\n{as_of}\n"}))
    assert len(found) == 1
    assert found[0].startswith(error)


URL, WEB, SCHEME, EMAIL = "a URL", "a web address", "a URL scheme", "an email address"
BRACKETS, LT = "square brackets are not allowed (links)", "`<` is not allowed (tags)"


@pytest.mark.parametrize(
    ("text", "whats"),
    [
        ("Read https://example.com first.", {URL}),
        ("Read HTTP://example.com first.", {URL}),
        ("Read www.evil.com first.", {WEB}),
        ("Mail mailto:a@b.c now.", {SCHEME, EMAIL}),
        ("Mail a@b.com now.", {EMAIL}),
        ("Fetch FTP://x now.", {SCHEME}),
        ("Run javascript:alert(1) now.", {SCHEME}),
        ("Read [the log](status.md).", {BRACKETS}),
        ("Read [the log][log].", {BRACKETS}),
        ("A shortcut [log] reference.", {BRACKETS}),
        ("Look: ![chart](chart.svg)", {BRACKETS}),
        ("[log]: status.md", {BRACKETS}),
        ("[a [b]](javascript:alert(1))", {BRACKETS, SCHEME}),
        ("[a\\]b](evil.com)", {BRACKETS}),
        ("> [x]: evil.com", {BRACKETS}),
        ("- [x]: evil.com", {BRACKETS}),
        ("Some <b>bold</b> text.", {LT}),
        ("A hidden <!-- comment -->.", {LT}),
        ("A <?php ?> instruction.", {LT}),
        ("12 < 1,234 is still a `<`.", {LT}),
    ],
)
def test_anything_but_plain_text_fails(text: str, whats: set[str]) -> None:
    found = offenders(text)
    assert {f.split(": ")[1] for f in found} == whats
    assert all(f.endswith("judgment blocks are plain text only") for f in found)


def test_each_rule_is_listed_once_per_block_with_its_first_hit() -> None:
    assert offenders("[a](b) and [c]") == [
        f"risks: {BRACKETS}: '[a](b) and [c]' (and 3 more); judgment blocks are plain text only"
    ]


def test_a_second_as_of_comment_is_a_tag() -> None:
    assert offenders(AS_OF) == [f"risks: {LT}: {AS_OF!r}; judgment blocks are plain text only"]


def test_plain_text_with_emphasis_lists_and_prose_colons_passes() -> None:
    text = "- **Bold** and _emphasis_; 12 is below 1,234.\n- The data: 12 functions, per file: 12."
    assert offenders(text) == []


@pytest.mark.parametrize("text", ["LOG10 and ATAN2 are off.", "Phase1 needs v15 and x_2."])
def test_digits_inside_identifiers_are_not_numbers(text: str) -> None:
    assert check.numbers(text) == []
    assert offenders(text) == []


def test_number_boundaries() -> None:
    assert check.numbers("1.5x, 1,234x, 97.5%, 1,234, and 12.") == ["1.5", "1,234", "97.5%", "1,234", "12"]
    assert check.numbers("500x faster, 99k users, 12ms, 3rd") == ["500", "99", "12", "3"]
    assert check.numbers("M2, M21, #1858, T-0074, SPEC-0022, decision 0118 and 2026-10-19.") == []


def test_identifier_digits_in_generated_blocks_are_not_cited() -> None:
    status = STATUS + " LOG10 is listed."
    assert check.offenders(a_page({"risks": f"\n{AS_OF}\n10 cells.\n"}, status=status)) == [
        "risks: the number 10 appears in no generated block"
    ]


def test_every_offender_is_listed_and_placeholders_fail() -> None:
    texts = {"bottom-line": blocks.PLACEHOLDER, "risks": f"\n{AS_OF}\n13 and 14, see https://x.y\n"}
    found = check.offenders(a_page(texts))
    assert [f.split(":")[0] for f in found] == ["bottom-line", "risks", "risks", "risks"]


def test_a_missing_block_fails() -> None:
    no_risks = a_page().replace(
        blocks.open_marker("judgment", "risks") + f"\n{AS_OF}\n" + blocks.close_marker("judgment"), ""
    )
    assert check.offenders(no_risks) == ["risks: the judgment block is missing"]


# A stale page as the generator writes it: the complete page of the 2026-10-19 run (AS_OF), then an
# incomplete run on 2026-10-20 generated at another HEAD. The banner also names the last complete
# run and the HEAD; only "The run of ..." is the run the as-of lines must name.
STALE_AS_OF = "<!-- as-of: 2026-10-20 def45678 -->"
LAST = {"date": "2026-10-19", "commit": "abc12345"}


def a_stale_page(as_of: str) -> str:
    texts = dict.fromkeys(page.JUDGMENT_BLOCKS, f"\n{as_of}\nSee the suites run (1) above.\n")
    complete = page.render(RUN, DATA, judgment_page(texts), SECTIONS)
    banner = page.stale_status("poi not fetched", "2026-10-20", "def45678", LAST, None, head="fedcba98")
    return page.render_stale(complete, banner)


def test_a_stale_page_whose_as_of_lines_name_the_banners_run_passes() -> None:
    assert check.offenders(a_stale_page(STALE_AS_OF)) == []


def test_a_stale_page_whose_as_of_lines_name_the_last_complete_run_fails() -> None:
    found = check.offenders(a_stale_page(AS_OF))
    assert found == [
        f"{name}: as-of 2026-10-19 abc12345 does not name the run in `status`, 2026-10-20 def45678"
        for name in page.JUDGMENT_BLOCKS
    ]


@pytest.mark.parametrize(
    "status",
    [
        "No run.",
        # A stale banner with no commit names no run an as-of line could match.
        page.stale_status("r", "2026-10-20", None, LAST, None, head=None),
        # Fullwidth digits are not ASCII: the date must be `[0-9]`, as RUN_DATE is.
        "The run of **\uff12\uff10\uff12\uff16-10-20** at commit `def45678`.",
    ],
)
def test_a_status_without_a_run_fails_loudly(status: str) -> None:
    with pytest.raises(ProgressReportError, match="names no run"):
        check.offenders(a_page(status=status))


def test_main_check_prints_ok_or_lists_offenders(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    (tmp_path / page.PAGE).write_text(a_page())
    progress_report.main(["--check", "--out-dir", str(tmp_path)])
    assert capsys.readouterr().out == "progress check ok\n"
    (tmp_path / page.PAGE).write_text(a_page({"risks": f"\n{AS_OF}\n99 cells.\n"}))
    with pytest.raises(SystemExit, match=r"1 offender.*\nrisks: the number 99 appears in no generated block"):
        progress_report.main(["--check", "--out-dir", str(tmp_path)])
    with pytest.raises(SystemExit, match="no page to check"):
        progress_report.main(["--check", "--out-dir", str(tmp_path / "none")])


def test_main_without_check_still_needs_out_dir() -> None:
    with pytest.raises(SystemExit):
        progress_report.main([])
