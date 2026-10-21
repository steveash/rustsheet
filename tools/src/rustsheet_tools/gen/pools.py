"""Edge pools for catalog parameters (`pool: <name>`, synthetic-generator.md §3.2 and §6.1).

Each entry is formula text. Nothing here may be a literal at or above 1E+308: Excel rejects the whole
workbook for one (docs/excel-semantics/numbers.md). `number_edges` is the targeted counterpart of the generic
`NUMBER_POOL` in generator.py, which every number literal draws from."""

from __future__ import annotations

from rustsheet_tools.gen.common import quoted
from rustsheet_tools.gen.number_formats import BUILTIN, CUSTOM

# The longest text a formula can hold as one string literal.
MAX_STRING_LITERAL = 255


def entries(text: str) -> list[str]:
    """The `|`-separated formula texts of `text` (a pool written compactly)."""
    return [e.strip() for e in text.split("|")]


POOLS: dict[str, list[str]] = {
    # Where number handling differs: 32- and 53-bit limits, the ends of the double range, values that do not
    # round-trip, the 1900 date-serial bounds.
    "number_edges": entries(
        "0 | 1 | (-1) | 2^31-1 | 2^31 | 2^32 | 2^53-1 | 2^53 | 2^53+1 | 999999999999999 | "
        "9999999999999999 | 1E+21 | 2.2250738585072E-308 | 1E-307 | 1E+307 | 0.1 | 0.3 | 1/3 | 2/3 | "
        "1+2^-52 | 1-2^-53 | 2.675 | 0.285 | (-2.5) | 0 | 59 | 60 | 61 | 2958465 | 2958466 | "
        "0.999988426"
    ),
    # ASIN, ACOS, ATANH (FISHER later): both ends of [-1, 1] and the first doubles past them.
    "unit_interval": entries(
        "(-1) | (-1-2^-52) | (-0.9999999999999999) | (-0.5) | (-1E-300) | 0 | 1E-300 | 0.5 | "
        "0.9999999999999999 | 1 | 1+2^-52 | 1.5"
    ),
    # SIN, COS, TAN, COT, SEC, CSC and the hyperbolics: multiples of pi, tiny angles, the large-argument
    # reduction limits (2^27 and beyond) and the overflow edge of SINH/COSH (710).
    "angle": entries(
        "PI() | (-PI()) | PI()/2 | PI()/4 | 3*PI()/2 | 2*PI() | 100*PI() | 1E-15 | 0.00625 | 28000 | "
        "54267 | 1E+8 | 134217727 | 134217728 | 1E+9 | 1E+15 | 1E+16 | 710 | (-710)"
    ),
    # CHAR, UNICHAR (BASE/DECIMAL radix later): control, ASCII, Latin-1 and code page gaps, surrogates and
    # the ends of the Unicode range.
    "code_point": entries(
        "0 | 1 | 9 | 10 | 31 | 32 | 65 | 127 | 128 | 129 | 141 | 143 | 144 | 157 | 160 | 255 | 256 | "
        "8364 | 55295 | 55296 | 57343 | 57344 | 65535 | 65536 | 1114111 | 1114112"
    ),
    # Text where case, width, normalization and whitespace rules differ (docs/excel-semantics/text.md).
    "text_edges": entries(
        '"" | " " | CHAR(9) | CHAR(10) | CHAR(160) | UNICHAR(12288) | UNICHAR(8203) | " 42 " | "a" | '
        '"A" | "ä" | "ß" | "İ" | "é" | "e"&UNICHAR(769) | UNICHAR(128512) | "\'a" | "a-b" | '
        'REPT("x",255) | REPT("ab",300) | "#N/A" | "=1+1"'
    ),
    # Text VALUE and NUMBERVALUE parse. No year-less date: "1/2" reads as a date in the current year, so its
    # value depends on the clock (synthetic-generator.md §6.2).
    "numeric_text": entries(
        '"42" | " 42 " | "4.2e1" | "1E3" | "1,000" | "$1,000.50" | "(100)" | "50%" | "-0" | "0x1F" | '
        '"TRUE" | "2024-02-29" | "1900-02-29" | "12:30" | "12:30 PM" | "25:00" | "1.5" | ".5" | '
        '"1e400"'
    ),
    # The format codes of the number-formats policy, as string literals (one definition: number_formats.py).
    "number_formats": [quoted(code) for code in (*BUILTIN, *CUSTOM) if len(code) <= MAX_STRING_LITERAL],
    # Error values, as literals and as computed (NA(), 1/0 and SQRT(-1) are what a workbook stores).
    "errors": entries(
        'NA() | 1/0 | #N/A | #DIV/0! | #VALUE! | #REF! | #NAME? | #NUM! | #NULL! | SQRT(-1) | VALUE("x")'
    ),
    # The date functions' serial arguments (docs/excel-semantics/dates.md): the 1900 bounds, "Serial 60 is the
    # fictitious 1900-02-29", "WEEKDAY of serial 0", "WORKDAY and NETWORKDAYS from serial 0", the last valid
    # serial (2958465 = 9999-12-31), fractional serials, 45351 (2024-02-29) and month- and year-end serials.
    "date_serial": entries(
        "0 | 1 | 59 | 60 | 61 | 366 | 367 | 1462 | 2958465 | 2958466 | (-1) | 0.5 | 59.999988426 | 60.5 | "
        "45000.75 | 45351 | 1E+15 | 31 | 365 | 44926 | 44957 | 45291 | 45382 | 73050"
    ),
    # HOUR, MINUTE, SECOND: "HOUR, MINUTE and SECOND round to the nearest second", the second before midnight,
    # half seconds, and "past 9999-12-31".
    "time_serial": entries(
        "0 | 0.5 | 0.999988426 | 0.99999999 | 1/86400 | 0.5/86400 | 1.5/86400 | 2958465.99999 | (-0.5)"
    ),
    # DATEVALUE: "The en-US date-text grammar" (month-first, month names, the 29 -> 2029 / 30 -> 1930 two-digit-year cut,
    # ISO, the fictitious 1900-02-29, an invalid day) and "Date text with a month name glued to digits". No
    # year-less date: it reads as a date in the current year, so its value depends on the clock.
    "date_text": entries(
        '"1/15/2024" | "01/05/2024" | "2/29/2024" | "2/30/2024" | "12-31-1999" | "2024-02-29" | '
        '"1900-02-29" | "1900-03-01" | "2024/3/5" | "1/15/29" | "1/15/30" | "12/31/99" | "March 5, 2020" | '
        '"1 July 2026" | "1-Jul-2026" | "1/Jul/26" | "Jul-2026" | "July 2026" | "SEPT 1, 2026" | '
        '"Jan2024" | "1JUN2021" | " 3/4/2020 " | "12/31/1899" | "12/31/9999" | "1/1/10000" | "13/1/2026"'
    ),
    # TIMEVALUE: "Time text" (12- and 24-hour, seconds, out-of-range hours, AM/PM, a one-letter suffix and
    # date-time text).
    "time_text": entries(
        '"1:30 PM" | "13:45:10" | "6:00" | "12:00 AM" | "12:00 PM" | "12:30:45 AM" | "0:00" | "23:59:59" | '
        '"24:00" | "25:00" | "9:15 a" | "9:15 p" | "11:12.5" | "10:11:12.1234" | "1/15/2024 18:30" | '
        '"2024-02-29 07:05:09" | "13:00 PM" | " 6:00 "'
    ),
    # NETWORKDAYS.INTL and WORKDAY.INTL weekend: 7-character masks (all weekend, none, short, long, not a
    # mask), numeric codes and the first codes past the table.
    "weekend_mask": entries(
        '"0000011" | "1000001" | "1111111" | "0000000" | "000001" | "00000011" | "abc" | 1 | 7 | 11 | 17 | 18'
    ),
    # WEEKDAY return_type: 0 (see "WEEKDAY of serial 0 and return_type 0"), 1-3, 11-17 and the first gap.
    "weekday_return": entries("1 | 2 | 3 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 0 | 4"),
    # WEEKNUM return_type: 1, 2, 11-17, the ISO system 21 and the first gap.
    "weeknum_return": entries("1 | 2 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 21 | 3"),
    # EDATE and EOMONTH months: fractions truncate toward zero, and 1200 / 120000 months reach the 9999 limit.
    "month_offset": entries("0 | 1 | (-1) | 1.5 | (-1.5) | 12 | 1200 | (-1200) | 120000"),
}
