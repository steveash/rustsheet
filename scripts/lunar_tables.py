#!/usr/bin/env python3
"""Probe and pack Excel's lunar calendars (locale tag calendar digits 11 and E) past 2100.

`case` writes probes/t2759-lunar-packed-<first year>, ten centuries each: one cell per century and table, whose value is the
lunar date of the century's first day, then one character per lunar month start in the century
(the gap in days from the previous start, or from the first day for the first start: chr(48+gap)
for a month other than 1, chr(80+gap) for month 1), then any day that breaks the rules the engine
relies on (day numbers count up by one within a month, months count up by one through a year and
the year turns only on the first day of month 1). Excel computes the cells (excel-graph).

`pack` turns the filled oracle file into formatter/lunar_2101.bin, the month lengths the engine reads
after its 1900-2100 tables (T-2759).
"""

from __future__ import annotations

import argparse
import bisect
import datetime as dt
import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CASES = ROOT / "eval/suites/probes/cases"
CASE_PREFIX = "t2759-lunar-packed"
# Centuries per case: Excel for the web aborts the recalculation of all 79 in one workbook.
CASE_ROWS = 10
BIN = ROOT / "crates/rustsheet-engine/src/formatter/lunar_2101.bin"
TABLES = (("A", "110409"), ("B", "E0409"))
FIRST_YEAR = 2101
LAST_YEAR = 9999
SPAN = 100
NEW_YEAR = 0x80
LENGTH = 0x1F
# Serials of 2101-01-01 and 9999-12-31.
FIRST_SERIAL = 73416
LAST_SERIAL = 2958465


def formula(tag: str, first: int, last: int) -> str:
    """The probe formula for one table over Gregorian years first..last."""
    f = f'"[$-{tag}]'
    return (
        f"=LET(a,DATE({first},1,1),s,SEQUENCE(DATE({last},12,31)-a+1,1,a),"
        f'y,--TEXT(s,{f}yyyy"),m,--TEXT(s,{f}m"),d,--TEXT(s,{f}d"),'
        f'yp,VSTACK(--TEXT(a-1,{f}yyyy"),DROP(y,-1)),'
        f'mp,VSTACK(--TEXT(a-1,{f}m"),DROP(m,-1)),'
        f'dp,VSTACK(--TEXT(a-1,{f}d"),DROP(d,-1)),'
        "b,(y<>yp)*((d<>1)+(m<>1)>0)+(d<>1)*((d<>dp+1)+(m<>mp)>0)"
        "+(d=1)*(m<>1)*(m<>mp+1)+(d=1)*(m=1)*(y<>yp+1),"
        "t,FILTER(HSTACK(s,m),d=1),v,TAKE(t,,1),"
        "g,v-VSTACK(a,DROP(v,-1)),"
        f'TEXT(a,{f}yyyy/m/d")&";"&CONCAT(IF(TAKE(t,,-1)=1,CHAR(80+g),CHAR(48+g)))'
        f'&";"&TEXTJOIN(",",,FILTER(s&":"&TEXT(s,{f}yyyy/m/d"),b>0,"")))'
    )


def spans() -> list[tuple[int, int]]:
    """Gregorian year ranges, one per probe row."""
    return [(y, min(y + SPAN - 1, LAST_YEAR)) for y in range(FIRST_YEAR, LAST_YEAR + 1, SPAN)]


def case_spans() -> list[tuple[str, list[tuple[int, int]]]]:
    """Each probe case's id and its rows' year ranges."""
    rows = spans()
    chunks = [rows[i : i + CASE_ROWS] for i in range(0, len(rows), CASE_ROWS)]
    return [(f"{CASE_PREFIX}-{chunk[0][0]}", chunk) for chunk in chunks]


def write_case() -> None:
    for case_id, rows in case_spans():
        cells: dict[str, str] = {}
        for row, (first, last) in enumerate(rows, start=1):
            for col, tag in TABLES:
                cells[f"{col}{row}"] = formula(tag, first, last)
        case = {
            "format": "rustsheet-case/v1",
            "description": (
                f"Chinese lunar calendars of calendar digits 11 (A) and E (B), {rows[0][0]}-"
                f"{rows[-1][1]}, one row per century: the lunar date of 1 January, one character "
                "per month start (gap from the previous start; chr(48+gap), or chr(80+gap) for "
                "month 1) and any day breaking the counting rules (T-2759, scripts/lunar_tables.py)."
            ),
            "tags": ["M10", "number-format", "probe"],
            "sheets": [{"name": "S", "cells": cells}],
        }
        (CASES / f"{case_id}.case.json").write_text(json.dumps(case, indent=1) + "\n")


def decode(value: str) -> tuple[tuple[int, int, int], list[tuple[int, bool]], list[str]]:
    """Split one cell into its first lunar date, (gap, is month 1) per start, and rule breaks."""
    # The codes can hold ";" (an 11-day gap), so the breaks are matched from the end.
    first, rest = value.split(";", 1)
    match = re.fullmatch(r"(.*);((?:\d+:\d+/\d+/\d+,?)*)", rest, re.DOTALL)
    if match is None:
        sys.exit(f"undecodable cell {value[:40]}...")
    codes, breaks = match.groups()
    y, m, d = (int(part) for part in first.split("/"))
    starts = [(ord(c) - 80, True) if ord(c) >= 80 else (ord(c) - 48, False) for c in codes]
    return (y, m, d), starts, [b for b in breaks.split(",") if b]


def day_number(serial: int) -> int:
    """The engine's day number of a serial past 1900-02-29 (lunar_calendar.rs)."""
    return serial - 1


def serial_of(year: int) -> int:
    """The serial of 1 January of `year`."""
    return FIRST_SERIAL + (dt.date(year, 1, 1) - dt.date(FIRST_YEAR, 1, 1)).days


def gregorian(serial: int) -> dt.date:
    return dt.date(1899, 12, 30) + dt.timedelta(days=serial)


def early_years(col: str, breaks: list[str]) -> list[tuple[int, int]]:
    """Day ranges that carry the next lunar year: from a 1 January on which the year turns
    mid-month to the day before the next month 1, on which it then does not turn. Any other break
    of the counting rules stops the packing."""
    ranges: list[tuple[int, int]] = []
    pending: tuple[int, int] | None = None
    for item in breaks:
        serial_text, lunar = item.split(":")
        serial = int(serial_text)
        y, m, d = (int(part) for part in lunar.split("/"))
        date = gregorian(serial)
        if pending is None and (date.month, date.day) == (1, 1) and (m, d) != (1, 1):
            pending = (serial, y)
        elif pending is not None and (m, d) == (1, 1) and y == pending[1]:
            ranges.append((day_number(pending[0]), day_number(serial) - 1))
            pending = None
        else:
            sys.exit(f"table {col}: unexpected break {item} ({date})")
    if pending is not None:
        sys.exit(f"table {col}: the year turned on 1 January {gregorian(pending[0])} but never at month 1")
    return ranges


def pack_table(col: str, cells: list[str]) -> bytes:
    """One table: first start day, its lunar year and month, month lengths, early years."""
    breaks: list[str] = []
    first_dates = []
    # Absolute start days and the month-counting year and month of each start. Each cell counts
    # its first gap from its own 1 January.
    days: list[int] = []
    labels: list[tuple[int, int]] = []
    for value, (first_year, _last) in zip(cells, spans()):
        first, cell_starts, cell_breaks = decode(value)
        first_dates.append(first)
        breaks.extend(cell_breaks)
        if not labels:
            year, month, _day = first
        day = day_number(serial_of(first_year))
        for gap, is_first_month in cell_starts:
            day += gap
            year, month = (year + 1, 1) if is_first_month else (year, month + 1)
            days.append(day)
            labels.append((year, month))
    early = early_years(col, breaks)
    check_repeated_days(col, days, breaks)
    if any(lo <= day_number(FIRST_SERIAL) <= hi for lo, hi in early):
        sys.exit(f"table {col}: 1 January {FIRST_YEAR} is in an early year")
    check_cell_starts(col, days, labels, early, first_dates)
    lengths = [b - a for a, b in zip(days, days[1:])] + [day_number(LAST_SERIAL) + 1 - days[-1]]
    body = bytearray()
    for index, length in enumerate(lengths):
        if not 0 < length <= LENGTH:
            sys.exit(f"table {col}: month {index} has {length} days")
        new_year = index + 1 < len(labels) and labels[index + 1][1] == 1
        body.append(length | (NEW_YEAR if new_year else 0))
    out = struct.pack("<IHBI", days[0], labels[0][0], labels[0][1], len(body)) + bytes(body)
    out += struct.pack("<B", len(early)) + b"".join(struct.pack("<II", lo, hi) for lo, hi in early)
    return out


def check_repeated_days(col: str, days: list[int], breaks: list[str]) -> None:
    """The engine's rule for the early years' 1 January: Excel's day number there is one less than
    the month start gives exactly when the month has 31 days (it repeats the day before), and no
    other month has 31 days."""
    long = {i for i, (a, b) in enumerate(zip(days, days[1:])) if b - a == 31}
    for item in breaks:
        serial_text, lunar = item.split(":")
        _y, m, d = (int(part) for part in lunar.split("/"))
        if (m, d) == (1, 1):
            continue
        day = day_number(int(serial_text))
        index = bisect.bisect_right(days, day) - 1
        shift = day - days[index] + 1 - d
        if shift != (index in long):
            sys.exit(f"table {col}: {item} is {shift} day(s) off the month start")
        long.discard(index)
    if long:
        sys.exit(f"table {col}: 31-day months away from an early year: {sorted(long)}")


def check_cell_starts(
    col: str,
    days: list[int],
    labels: list[tuple[int, int]],
    early: list[tuple[int, int]],
    first_dates: list[tuple[int, int, int]],
) -> None:
    """Each century cell's own first lunar date must follow from the months before it."""
    for (first, _last), date in zip(spans(), first_dates):
        a = day_number(serial_of(first))
        index = bisect.bisect_right(days, a) - 1
        if index < 0:
            continue
        year, month = labels[index]
        if any(lo <= a <= hi for lo, hi in early):
            year += 1
        if (year, month, a - days[index] + 1) != date:
            sys.exit(f"table {col}: 1 January {first} reads {date}, the months give {(year, month)}")


def pack() -> None:
    values: dict[str, list[str]] = {col: [] for col, _ in TABLES}
    for case_id, rows in case_spans():
        oracle = json.loads((CASES / f"{case_id}.oracle.json").read_text())["sheets"]["S"]
        for col, _ in TABLES:
            values[col] += [oracle[f"{col}{row}"]["v"] for row in range(1, len(rows) + 1)]
    BIN.write_bytes(b"".join(pack_table(col, values[col]) for col, _ in TABLES))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["case", "pack"])
    if parser.parse_args().command == "case":
        write_case()
    else:
        pack()


if __name__ == "__main__":
    main()
