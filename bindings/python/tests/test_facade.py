"""Tests for the typed facade API (``rustsheet.Workbook``)."""

import threading
import time

import pytest
import rustsheet as rs


@pytest.fixture
def wb() -> rs.Workbook:
    return rs.Workbook()


def test_empty_workbook_has_one_sheet(wb):
    assert wb.sheet_names == ["Sheet1"]
    assert wb["Sheet1!A1"] is None


def test_edit_commits_and_reports_changes(wb):
    with wb.edit() as tx:
        tx["Sheet1!A1"] = 10
        tx["Sheet1!A2"] = rs.Formula("=A1*2")
    assert wb["Sheet1!A2"] == 20.0
    assert wb.changes is not None
    refs = {c["ref"]: c for c in wb.changes["values"]}
    assert refs["Sheet1!A2"]["after"] == 20.0
    assert refs["Sheet1!A2"]["before"] is None
    assert {c["ref"] for c in wb.changes["inputs"]} == {"Sheet1!A1", "Sheet1!A2"}
    assert wb.formula("Sheet1!A2") == "=A1*2"


def test_changes_is_none_before_any_edit(wb):
    assert wb.changes is None


def test_exception_in_block_rolls_back(wb):
    wb.set("Sheet1!A1", 1)
    with pytest.raises(RuntimeError):
        with wb.edit() as tx:
            tx["Sheet1!A1"] = 99
            tx.add_sheet("Extra")
            raise RuntimeError("boom")
    assert wb["Sheet1!A1"] == 1.0
    assert wb.sheet_names == ["Sheet1"]


def test_failing_op_rolls_back_whole_transaction(wb):
    wb.set("Sheet1!A1", 1)
    with pytest.raises(rs.EditError):
        with wb.edit() as tx:
            tx["Sheet1!A1"] = 5
            tx.rename_sheet("Sheet1", "")  # invalid sheet name
    assert wb["Sheet1!A1"] == 1.0
    assert wb.sheet_names == ["Sheet1"]


def test_unknown_sheet_is_ref_error(wb):
    with pytest.raises(rs.RefError):
        wb["Nope!A1"]
    with pytest.raises(rs.RefError):
        with wb.edit() as tx:
            tx.delete_sheet("Nope")


def test_error_hierarchy():
    for exc in (rs.RefError, rs.EditError, rs.LoadError, rs.SaveError):
        assert issubclass(exc, rs.RustSheetError)


def test_excel_errors_are_values_not_exceptions(wb):
    wb.set("Sheet1!A1", "=1/0")
    err = wb["Sheet1!A1"]
    assert isinstance(err, rs.XlError)
    assert err == rs.XlError("#DIV/0!")
    assert str(err) == "#DIV/0!"
    assert err.code == 2
    assert err != rs.XlError("#N/A")
    assert len({err, rs.XlError("#DIV/0!")}) == 1


@pytest.mark.parametrize(
    ("literal", "code"),
    [
        ("#GETTING_DATA", 8),
        ("#CONNECT!", 10),
        ("#BLOCKED!", 11),
        ("#FIELD!", 13),
        ("#BUSY!", 0),
    ],
)
def test_newer_error_kinds(literal, code):
    # SPEC-0003: XlError accepts the stored names, including ones that are not formula literals.
    err = rs.XlError(literal)
    assert err.code == code
    assert str(err) == literal
    assert err == rs.XlError(literal.lower())


@pytest.mark.parametrize(
    ("literal", "code"),
    [
        ("#GETTING_DATA", 8),
        ("#CONNECT!", 10),
        ("#BLOCKED!", 11),
        ("#FIELD!", 13),
        ("#BUSY!", 0),
    ],
)
def test_set_stores_typed_error_cells(wb, literal, code):
    # These are not typeable literals, so Workbook.set must not parse them as text (T-1145).
    wb.set("Sheet1!A1", rs.XlError(literal))
    wb.set("Sheet1!B1", "=ERROR.TYPE(A1)")
    assert wb["Sheet1!A1"] == rs.XlError(literal)
    assert wb["Sheet1!B1"] == code


def test_getting_data_formula_reads_back(wb):
    wb.set("Sheet1!A1", "=#GETTING_DATA")
    assert wb["Sheet1!A1"] == rs.XlError("#GETTING_DATA")


def test_unknown_error_literal_rejected():
    with pytest.raises(TypeError):
        rs.XlError("#BOGUS")


def test_str_input_parses_like_excel_typing(wb):
    wb.set("Sheet1!A1", "12%")
    wb.set("Sheet1!A2", "=A1+1")
    wb.set("Sheet1!A3", "'=text")
    wb.set("Sheet1!A4", "hello")
    assert wb["Sheet1!A1"] == pytest.approx(0.12)
    assert wb["Sheet1!A2"] == pytest.approx(1.12)
    assert wb["Sheet1!A3"] == "=text"
    assert wb["Sheet1!A4"] == "hello"


def test_typed_inputs(wb):
    wb.set("Sheet1!A1", True)
    wb.set("Sheet1!A2", 3)
    wb.set("Sheet1!A3", 2.5)
    wb.set("Sheet1!A4", rs.XlError("#N/A"))
    assert wb["Sheet1!A1"] is True
    assert wb["Sheet1!A2"] == 3.0
    assert wb["Sheet1!A3"] == 2.5
    assert wb["Sheet1!A4"] == rs.XlError("#N/A")
    wb.set("Sheet1!A2", None)
    assert wb["Sheet1!A2"] is None


def test_unsupported_input_type(wb):
    with pytest.raises(TypeError):
        wb.set("Sheet1!A1", [1, 2])


def test_values_returns_rows(wb):
    with wb.edit() as tx:
        tx["A1"] = 1
        tx["B1"] = "x"
        tx["A2"] = True
    assert wb.values("Sheet1!A1:B2") == [[1.0, "x"], [True, None]]


def test_values_too_large_is_ref_error(wb):
    with pytest.raises(rs.RefError):
        wb.values("A:XFD")


def test_sheet_ops_in_one_transaction(wb):
    with wb.edit() as tx:
        tx.add_sheet("Data")
        tx["Data!A1"] = 7
        tx["Sheet1!A1"] = rs.Formula("=Data!A1+1")
        tx.rename_sheet("Data", "Numbers")
    assert wb.sheet_names == ["Sheet1", "Numbers"]
    assert wb["Sheet1!A1"] == 8.0
    kinds = [s["kind"] for s in wb.changes["sheets"]]
    assert kinds == ["added", "renamed"]
    with wb.edit() as tx:
        tx.delete_sheet("numbers")  # case-insensitive
    assert wb.sheet_names == ["Sheet1"]


def test_set_track_changes_off_empties_value_diff(wb):
    wb.set_track_changes(False)
    wb.set("A1", 1)
    assert wb.changes["values"] == []
    assert len(wb.changes["inputs"]) == 1


def test_formatted_and_number_format(wb):
    wb.set("A1", "12%")
    assert wb.formatted("A1") == "12%"
    assert wb.number_format("A1") is not None
    assert wb.number_format("A2") is None


def test_used_range(wb):
    assert wb.used_range("Sheet1") is None
    wb.set("B2", 1)
    assert wb.used_range("Sheet1") == "Sheet1!B2"


def test_transaction_is_single_use(wb):
    tx = wb.edit()
    with tx:
        tx["A1"] = 1
    with pytest.raises(rs.EditError):
        tx["A1"] = 2


def test_save_and_open_roundtrip(wb, tmp_path):
    with wb.edit() as tx:
        tx["A1"] = 4
        tx["A2"] = rs.Formula("=A1^2")
    path = tmp_path / "book.xlsx"
    wb.save_xlsx(str(path))
    reopened = rs.open(str(path))
    assert reopened["Sheet1!A2"] == 16.0
    again = rs.Workbook.from_bytes(wb.to_xlsx_bytes())
    assert again["Sheet1!A2"] == 16.0


def test_open_missing_file_is_load_error(tmp_path):
    with pytest.raises(rs.LoadError):
        rs.Workbook.open(str(tmp_path / "missing.xlsx"))


def test_from_bytes_garbage_is_load_error():
    with pytest.raises(rs.LoadError):
        rs.Workbook.from_bytes(b"not a zip")


def test_gil_released_during_recalculate(wb):
    """Another Python thread keeps ticking *during* a long recalculation (GIL released)."""
    with wb.edit() as tx:
        tx["A1"] = 1
        for r in range(2, 20001):
            tx[f"A{r}"] = rs.Formula(f"=A{r - 1}+1")
    ticks = 0
    stop = threading.Event()

    def spin():
        nonlocal ticks
        while not stop.is_set():
            ticks += 1
            time.sleep(0.0005)

    t = threading.Thread(target=spin)
    t.start()
    try:
        time.sleep(0.05)
        before = ticks
        start = time.monotonic()
        while time.monotonic() - start < 0.5:
            wb.recalculate()
        during = ticks - before
    finally:
        stop.set()
        t.join()
    # A ticker starved by a GIL-holding recalculation would count ~0 ticks over 0.5s.
    assert during >= 20


def test_xlerror_detail_and_deleted_sheet_changes(wb):
    wb.set("A1", "=1/0")
    assert wb["A1"].detail is None or isinstance(wb["A1"].detail, str)
    with wb.edit() as tx:
        tx.add_sheet("Tmp")
    with wb.edit() as tx:
        tx.delete_sheet("Tmp")
    assert wb.changes["sheets"] == [{"kind": "deleted", "name": "Tmp"}]


def test_moderately_nested_formula(wb):
    wb.set("A1", rs.Formula("=" + "(" * 100 + "1" + ")" * 100))
    assert wb["A1"] == 1.0


def test_too_deeply_nested_formula_is_rejected(wb):
    # The parser bounds the depth (MAX_FORMULA_DEPTH, T-0931): the edit fails and rolls back.
    wb.set("A1", 7)
    with pytest.raises(rs.EditError):
        wb.set("A1", rs.Formula("=" + "(" * 10_000 + "1" + ")" * 10_000))
    assert wb["A1"] == 7.0


def test_reads_do_not_deadlock_with_concurrent_recalc_and_save(wb, tmp_path):
    """Reads in one thread while another recalculates/saves must finish (GIL vs mutex order)."""
    with wb.edit() as tx:
        tx["A1"] = 1
        for r in range(2, 5001):
            tx[f"A{r}"] = rs.Formula(f"=A{r - 1}+1")
    path = str(tmp_path / "busy.xlsx")
    errors = []

    def writer():
        try:
            for _ in range(5):
                wb.recalculate()
                wb.save_xlsx(path)
        except Exception as e:  # noqa: BLE001
            errors.append(e)

    t = threading.Thread(target=writer, daemon=True)
    t.start()
    for _ in range(200):
        assert wb["Sheet1!A1"] == 1.0
        wb.values("A1:A3")
        wb.sheet_names
    t.join(timeout=60)
    assert not t.is_alive(), "deadlock between GIL and workbook mutex"
    assert not errors


def test_huge_int_is_type_error(wb):
    with pytest.raises(TypeError):
        wb.set("A1", 10**400)


def test_date_system_defaults_to_1900_and_round_trips_1904():
    assert rs.Workbook().date_system == "1900"
    wb = rs.Workbook(date_system="1904")
    assert wb.date_system == "1904"
    assert rs.Workbook.from_bytes(wb.to_xlsx_bytes()).date_system == "1904"
    with pytest.raises(TypeError):
        rs.Workbook(date_system="1999")


DAY_MS = 86_400_000


def _clock_book(clock=0):
    wb = rs.Workbook(clock=clock)
    wb.set("Sheet1!A1", rs.Formula("=NOW()"))
    wb.set("Sheet1!B1", rs.Formula("=TODAY()"))
    wb.set("Sheet1!A2", rs.Formula("=A1+1"))
    return wb


def test_fixed_clock_drives_now_and_today_and_survives_reload():
    wb = _clock_book()
    assert wb.clock == 0
    assert wb["Sheet1!A1"] == 25569.0
    assert wb["Sheet1!B1"] == 25569.0
    loaded = rs.Workbook.from_bytes(wb.to_xlsx_bytes(), clock=DAY_MS)
    assert loaded["Sheet1!A1"] == 25570.0
    assert loaded.clock == DAY_MS


def test_setting_the_clock_is_one_edit_with_changes():
    wb = _clock_book()
    wb.clock = DAY_MS
    assert wb["Sheet1!A1"] == 25570.0
    assert wb["Sheet1!B1"] == 25570.0
    assert wb["Sheet1!A2"] == 25571.0
    assert wb.changes is not None
    assert {c["ref"] for c in wb.changes["values"]} == {"Sheet1!A1", "Sheet1!A2", "Sheet1!B1"}
    assert wb.changes["inputs"] == []
    wb.clock = DAY_MS
    assert wb.changes["values"] == []


def test_clock_none_restores_the_system_clock():
    wb = _clock_book()
    wb.clock = None
    assert wb.clock is None
    assert wb["Sheet1!A1"] > 45000.0  # the real now, long after 2023


@pytest.mark.parametrize("bad", [-2_208_988_800_001, 253_402_300_800_000, 2**70])
def test_clock_out_of_range_is_value_error(bad):
    with pytest.raises(ValueError):
        rs.Workbook(clock=bad)
    wb = _clock_book()
    with pytest.raises(ValueError):
        wb.clock = bad
    assert wb.clock == 0


@pytest.mark.parametrize("bad", [True, 1.5, "0"])
def test_clock_wrong_type_is_type_error(bad):
    with pytest.raises(TypeError):
        rs.Workbook(clock=bad)
    wb = _clock_book()
    with pytest.raises(TypeError):
        wb.clock = bad


def test_lone_surrogate_reads_back_exactly(wb):
    wb.set("A1", rs.Formula('=MID("a😀b",3,1)'))
    wb.set("A2", "�")
    wb.set("A3", "x")
    assert wb.value("A1") == "\ude00"
    assert wb.values("A1:A3") == [["\ude00"], ["�"], ["x"]]
    assert wb.values("A2") == [["�"]]
    assert wb.value("A2") == "�"
