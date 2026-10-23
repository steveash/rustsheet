"""Edge pools for catalog parameters (`pool: <name>`, synthetic-generator.md §3.2 and §6.1).

Each entry is formula text. Nothing here may be a literal at or above 1E+308: Excel rejects the whole
workbook for one (docs/excel-semantics/numbers.md). `number_edges` is the targeted counterpart of the generic
`NUMBER_POOL` in generator.py, which every number literal draws from."""

from __future__ import annotations

import re

from rustsheet_tools.gen.common import quoted
from rustsheet_tools.gen.number_formats import BUILTIN, CUSTOM

# The longest text a formula can hold as one string literal.
MAX_STRING_LITERAL = 255


def entries(text: str, sep: str = "|") -> list[str]:
    """The `sep`-separated formula texts of `text` (a pool written compactly)."""
    return [e.strip() for e in text.split(sep)]


POOLS: dict[str, list[str]] = {
    # Where number handling differs: 32- and 53-bit limits, the ends of the double range, values that do not
    # round-trip, the 1900 date-serial bounds.
    "number_edges": entries(
        "0 | 1 | (-1) | 2^31-1 | 2^31 | 2^32 | 2^53-1 | 2^53 | 2^53+1 | 999999999999999 | "
        "9999999999999999 | 1E+21 | 2.2250738585072E-308 | 1E-307 | 1E+307 | 0.1 | 0.3 | 1/3 | 2/3 | "
        "1+2^-52 | 1-2^-53 | 2.675 | 0.285 | (-2.5) | 0 | 59 | 60 | 61 | 2958465 | 2958466 | "
        "0.999988426"
    ),
    # ASIN, ACOS, ATANH, FISHER: both ends of [-1, 1] and the first doubles past them.
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
    # The shape-driven array functions (TAKE, DROP, EXPAND, CHOOSEROWS/COLS, WRAPROWS/COLS, the stacks, ROWS,
    # COLUMNS) over every shape docs/excel-semantics/arrays.md separates: a single value and a 1x1 array or range
    # ("WRAPCOLS of a single value is not padded", "INDEX of a scalar or a 1x1 array", "BYROW and BYCOL of a single
    # value"), a 1xn row, an nx1 column, 2-D arrays, and ragged results the stacks pad with #N/A ("Stacked results
    # past the grid"; EXPAND's #N/A padding).
    "array_shape": entries(
        '5 | "a" | {1} | SEQUENCE(1,1) | Data!A1 | {1,2,3} | {1;2;3} | Data!A1:E1 | Data!A1:A12 | {1,2;3,4} | '
        '{1,2,3;4,5,6} | {"a","b";"c","d"} | VSTACK({1,2},{3}) | HSTACK({1;2},3) | EXPAND({1},2,3)'
    ),
    # Arrays holding errors and blanks: error elements broadcast or are kept per element ("A scalar error
    # operand broadcasts", "SCAN and REDUCE keep errors in the LAMBDA"), TOCOL/TOROW's ignore drops blanks and
    # errors, and a blank cell is handed on as a reference, not as 0 ("BYROW, BYCOL, MAP, SCAN and REDUCE hand a
    # range's slices to the LAMBDA as references"). Column Z of Data is never filled, so Data!Z1:Z3 is blank.
    "array_holes": entries(
        '{1,#N/A,3} | {#DIV/0!;2;#VALUE!} | {"",0,FALSE} | {1,2;#N/A,""} | IF({1,0,1},1,NA()) | '
        'IFERROR(1/{1,0,2},"") | Data!Z1 | Data!Z1:Z3 | HSTACK(Data!Z1:Z2,{1;2}) | EXPAND(1,2,2,"") | '
        '{TRUE,FALSE;"TRUE",1}'
    ),
    # A value looked up: where the type-skipping binary search and the wildcard and next-smaller/larger modes
    # change their answer (lookup.md "Approximate-match lookups: type-skipping binary search", "XMATCH and
    # XLOOKUP next-smaller mode: text sorts above every number"): numbers below, between and above the data,
    # text in either case, wildcards and an escaped one (match_mode 2), booleans, number-looking text, an empty
    # string and an error.
    "lookup_value": entries(
        '0 | 1 | 2.5 | (-1) | 1E+300 | 45351 | "" | "a" | "A" | "b" | "zzz" | "1" | "*" | "a*" | "?" | "~*" | '
        "TRUE | FALSE | NA()"
    ),
    # The vector LOOKUP and XMATCH search: sorted, descending, unsorted, runs of duplicates (lookup.md "XMATCH /
    # XLOOKUP binary search over runs of equal values"), mixed types, text case, an error element, blanks
    # ("XMATCH / XLOOKUP linear search over blanks"), a single element and a row instead of a column. No 2-D
    # table: XMATCH's lookup_array shares the pool, and a 2-D one is only #VALUE! there.
    "lookup_table": entries(
        '{1;2;3;4;5} | {5;4;3;2;1} | {3;1;4;1;5} | {1;2;2;2;3} | {1;"a";TRUE;2;"b"} | {"a";"B";"c";"D"} | '
        "{1;#N/A;3} | Data!Z1:Z4 | {1} | {1,2,3}"
    ),
    # REGEXTEST, REGEXEXTRACT and REGEXREPLACE patterns (PCRE2; text.md's REGEX entries): anchors, the empty
    # pattern and patterns that match empty (the iteration after an empty match), alternation with an empty
    # side, a lookahead, inline (?i), (?m) and (?s), POSIX and Unicode classes, a named group and its
    # back-reference, \X and \R and a counted repeat. No invalid pattern: excel-graph's error for "[" is a rich
    # value it cannot decode, so the cell would stay pending (T-2286 smoke check).
    # (Separated by spaces: the patterns hold `|` and no space.)
    "regex_pattern": entries(
        '"^" "$" "^$" "" "\\b" "a*?" "x*" "|b" "b|" "(?=b)|b" "(?i)A" '
        '"(?m)^a" "(?s).+" "[[:alpha:]]+" "\\d+" "\\p{L}" "[^a-z]" "(a)|b" "(?<n>\\w)\\k<n>" "\\X" "\\R" "a{2,}" "." "\\."',
        sep=" ",
    ),
    # LAMBDAs for MAP, BYROW, BYCOL, REDUCE, SCAN and MAKEARRAY, by parameter count (the generator takes only
    # entries of the arity the call needs, and renames x and y to free names): an identity that hands back a
    # reference or a row (BYROW's #CALC!, "BYROW and BYCOL: a LAMBDA result that is an array"), an array or a
    # multi-cell range result ("MAP and SCAN with a LAMBDA that returns a multi-cell range"), an error ("SCAN
    # and REDUCE keep errors in the LAMBDA"), ISBLANK of a handed-on blank reference, a LAMBDA result ("An
    # uncalled LAMBDA is one value"), REDUCE's VSTACK accumulator, and an omitted optional parameter.
    "lambda_body": entries(
        'LAMBDA(x,x) | LAMBDA(x,SUM(x)) | LAMBDA(x,NA()) | LAMBDA(x,"") | LAMBDA(x,ISBLANK(x)) | '
        'LAMBDA(x,ROWS(x)) | LAMBDA(x,{1,2}) | LAMBDA(x,x&"") | LAMBDA(x,IF(x>1,x,1/0)) | '
        "LAMBDA(x,LAMBDA(y,y)) | LAMBDA(x,Data!A1:B2) | LAMBDA(x,y,x+y) | LAMBDA(x,y,y) | LAMBDA(x,y,x&y) | "
        "LAMBDA(x,y,IF(y>x,y,x)) | LAMBDA(x,y,NA()) | LAMBDA(x,y,{1,2}) | LAMBDA(x,y,ISBLANK(y)) | "
        'LAMBDA(x,y,IFERROR(x/y,"e")) | LAMBDA(x,y,VSTACK(x,y)) | LAMBDA(x,y,x*y) | LAMBDA(x,[y],ISOMITTED(y))'
    ),
    # Probabilities of the distributions and their inverses, the alphas of BINOM.INV, CRITBINOM and CONFIDENCE,
    # PERCENTILE's k and TRIMMEAN's percent (docs/excel-semantics/statistics.md): both ends of [0, 1]; the tiny
    # and near-1 values the inverse CDFs still answer ("Inverse CDF functions at a tiny or near-1 probability":
    # 1-1E-16 is a double below 1, 1-1E-17 is 1 and gives #NUM!; "Student-t quantile at a tiny probability":
    # T.INV(1E-300,1) is finite); the usual test levels; and the first values outside the interval.
    "probability": entries(
        "0 | 1 | 1E-300 | 1E-200 | 1E-100 | 1E-20 | 1E-16 | 1E-10 | 0.001 | 0.025 | 0.05 | 0.5 | 0.95 | "
        "0.975 | 0.999999 | 1-1E-10 | 1-1E-16 | 1-1E-17 | (-1E-300) | (-0.5) | 1+2^-52 | 1.5"
    ),
    # Degrees of freedom, shapes (BETA, GAMMA, WEIBULL alpha and beta), EXPON's lambda, POISSON's mean and the
    # standard deviations (statistics.md): 1 and 2 ("T.DIST.RT at df 2 is BETA.DIST's closed form"), values that
    # truncate ("df is truncated, not rounded": 1.999 is df 1), below 1 (#NUM!, or the t pdf's #DIV/0!), 0 and
    # negatives, tiny shapes (BETA.INV(p,0.01,0.01) is #NUM!), 1E5 (where our kernels switch method) and the
    # df cut: 1E10 and 10000000000.5 work, 10000000001 and 1.5E10 are #NUM! ("F-distribution df limit",
    # "Upper df limit").
    "shape": entries(
        "1 | 2 | 3 | 0.5 | 1.5 | 2.5 | 0.999 | 1.999 | 0.01 | 1E-10 | 1E-300 | 0 | (-0.5) | (-1) | 30 | 100 | "
        "99999 | 1E+5 | 1E+8 | 1E+10 | 10000000000.5 | 10000000001 | 1.5E+10"
    ),
    # x and z of the distributions: 0, the tiny values either side of it, values near the mode and median of the
    # usual shapes, the normal's deep tails (its density underflows past about 38; "The normal CDF is not always
    # correctly rounded"), large values and the overflowing products of "Overflowing arguments, PERMUT's limit
    # and argument error order" (F.DIST's deg_freedom1 * x). Nothing at 1E+308: a literal there is refused.
    "stat_x": entries(
        "0 | 1E-300 | (-1E-300) | 1E-10 | 0.5 | 1 | (-1) | 1.5 | 2 | (-2) | 3 | 5 | 10 | 37.5 | 38.5 | (-38.5) | "
        "40 | (-40) | 100 | 1E+5 | 1E+10 | 1E+300 | (-1E+300) | 1E+307"
    ),
    # GAMMA, GAMMALN and GAMMALN.PRECISE ("GAMMA and GAMMALN sweep"): exact small integers, 25 and 26 (where a
    # factorial table stops matching), the poles at 0 and the negative integers, values between poles, the
    # overflow edge (171.6 is finite, 171.7 is #NUM!) and GAMMALN, which has no upper limit.
    "gamma_x": entries(
        "1 | 2 | 3 | 0.5 | 1E-300 | 1E-10 | 0 | (-1) | (-0.5) | (-1.5) | (-170.5) | 25 | 26 | 171 | 171.5 | "
        "171.6 | 171.7 | 172 | 200 | 1E+300"
    ),
    # The cumulative flag of the distributions and LINEST/LOGEST/GROWTH/TREND's const and stats: booleans as
    # TRUE and FALSE, as 0, 1 and other numbers, as text (logical text and other text: "Thin distribution
    # functions: argument coercion, truncation and error edges"), and an error.
    "flag": entries(
        'TRUE | FALSE | 0 | 1 | 2 | (-1) | 0.5 | "TRUE" | "FALSE" | "true" | "1" | "x" | "" | NA()'
    ),
    # Data of the statistical aggregates, the order statistics and the paired functions: one value as a scalar
    # and as an array ("Array arguments given a scalar, a single cell or an error"), constant values (zero
    # variance), two values ("SKEW.P of two numbers is #DIV/0!"), text, booleans and blanks inside an array
    # ("VARA family: single-cell reference arguments", "AVERAGEA, MAXA and MINA over an array"; Data!Z is
    # never filled), an error element, a big common offset and values that cancel ("VAR/STDEV switch between a
    # one-pass and a two-pass formula", "Paired regression functions: ... large offsets"; {0.02,0.027} takes
    # the two-pass form), squares that overflow, negatives and zeros (GEOMEAN, HARMEAN), ties (MODE, RANK) and a
    # row instead of a column (paired sizes that differ are #N/A).
    "stat_data": entries(
        '5 | {5} | {3,3,3,3} | {1,2} | {1;2;3;4;5} | {1,2,3,4,5,6,7,8,9,10} | {1,"a",TRUE,3} | {1;2;"";4} | '
        "{TRUE,FALSE,1} | Data!Z1:Z3 | {1,#N/A,3} | {100000000.1,100000000.2,100000000.3} | {1E+15,1,-1E+15} | "
        "{1E+200,1E+200,-1E+200} | {0.02,0.027} | {-1,0,1} | {0,0,0} | {1,1,2,2,2,3} | Data!A1:A12 | Data!A1:E1"
    ),
    # The M9 financial functions (docs/excel-semantics/financial.md). Rates, guesses and yields: 0 (PV, FV, PMT
    # and NPER take their rate-0 branch), -1 (1+rate is 0) and the first doubles above it, tiny rates that 1+rate
    # rounds away ("CUMIPMT and CUMPRINC over many periods": a rate of 1E-300 is not lost), the usual monthly and
    # yearly rates, 1 and 100 where (1+rate)^nper overflows ("PMT when (1+rate)^nper overflows or underflows":
    # PMT(1,2000,1), PMT(100,184,439103)), negatives (a negative coupon or price is #NUM!) and 1E+10
    # (CUMIPMT(1E10,100,1,1,100,0)); 1E+300 overflows EFFECT's power ("EFFECT and NOMINAL raise to their power").
    "rate": entries(
        "0 | (-1) | (-0.9999999999) | (-1+2^-52) | (-0.5) | (-0.05) | 1E-300 | 1E-10 | 1E-9 | 0.05/12 | 0.05 | "
        "0.1 | 0.5 | 1 | 10 | 100 | 1E+10 | 1E+300"
    ),
    # nper and an asset's life: 0 (PMT(0.05,0,1000) is #NUM!), fractions (CUMIPMT's end period below nper + 1, an
    # integer-power path that no longer applies), the 1023/1024 and 2000 periods where (1+rate)^nper overflows at a
    # rate of 1 ("PMT when (1+rate)^nper overflows"), 1E+6 and 1E+9 (constant-time CUMIPMT, the CUMIPMT, DB and
    # VDB caps), negatives (PMT(1,-2000,1) is #NUM!; PMT(0.05,-12,1000) computes) and 1E+300.
    "nper": entries(
        "0 | 1 | 0.5 | 1.5 | 12.5 | 360 | 1023 | 1024 | 2000 | 1E+6 | 1E+9 | (-1) | (-12) | 1E+300"
    ),
    # A period inside nper (IPMT and PPMT per, DB and DDB period, CUMIPMT's and VDB's start and end): 0 and
    # negatives (#NUM!), 1, fractions (CUMIPMT rounds the start up and the end down; VDB's charge order), and
    # periods past the domain's nper (an end period past nper is #NUM!, "DB and VDB period cap").
    "period": entries("0 | 1 | 2 | 0.5 | 1.5 | 1.999 | (-1) | 12 | 13 | 360 | 361 | 1E+6"),
    # pv, fv, pmt and the depreciation and securities amounts: zeros (PMT(1,2000,0,5) is 0), both signs of one
    # value (an annuity needs pv and fv of opposite signs for NPER and RATE; independent draws give every sign
    # combination), the overflow edge of pv*(1+rate)^nper (PMT(100,184,439103)), a tiny value and +-1E+300.
    "cash_amount": entries(
        "0 | 1 | (-1) | 0.01 | 100 | (-100) | 1000 | (-1000) | 439103 | (-439103) | 1E-300 | 1E+300 | (-1E+300)"
    ),
    # A security's price, redemption and par percentage: 0 and negatives (#NUM!), par itself and the values just
    # either side of it, a deep discount and premium, and 1E+300 (YIELD's slowest solve, "Yield and odd-coupon work
    # cap").
    "price": entries("0 | (-1) | 0.01 | 50 | 95.04 | 99.999 | 100 | 100.001 | 105 | 200 | 1E+6 | 1E+300"),
    # The annuity `type` (payment at the start or end of a period): 0 and 1, other numbers, fractions and negatives
    # (any nonzero is the start: PV at 2, -1 and 0.5 equals PV at 1), booleans, number text (coerced) and other
    # text (#VALUE!) ("Financial edge-pool smoke check (T-2248)").
    "annuity_type": entries('0 | 1 | 2 | (-1) | 0.5 | 1.5 | TRUE | FALSE | "1" | "x"'),
    # Settlement, maturity, issue and the coupon and purchase dates: serial 0, 0.5 and -1 ("Coupon settlement
    # before the calendar": PRICE is #NUM! below serial 1, the COUP* functions only below 0), serial 1 and the
    # fictitious 1900-02-29 at 60 with its neighbours ("Coupon schedules at the calendar's edges": COUPDAYBS(60,...)
    # is -1), leap days and February ends of 2000, 2007, 2008, 2020, 2023 and 2024 ("COUPDAYS basis 1 around 29
    # February"), month ends (31st, 30th, 31 December: month-end coupon schedules), the last serial 2958465 and
    # the first past it, a fractional serial (it truncates) and a text date. Settlement and maturity draw from the
    # same pool, so the same day and the wrong order both occur. Kept apart from `date_serial`: a bond needs
    # coupon-schedule dates (month ends, February ends) and a text date, not time fractions or the 1E+15 serial.
    "security_date": entries(
        "0 | 0.5 | (-1) | 1 | 59 | 60 | 61 | 366 | 36584 | 36585 | 36586 | 39141 | 39448 | 39507 | 43830 | "
        "43831 | 43890 | 44985 | 45322 | 45351 | 45412 | 45535 | 45657 | 47664 | 48638 | 45351.75 | 2958465 | "
        '2958466 | "2024-02-29"'
    ),
    # The day-count basis: 0-4, the first values outside (5 and -1 are #NUM!, and any negative: "Day-count basis:
    # a negative basis is #NUM!"), fractions that truncate (4.9 is 4, -0.5 is #NUM!), number text (coerced) and a
    # boolean (#VALUE!: "Discount-security functions: settlement 0, basis and argument modes").
    "basis": entries('0 | 1 | 2 | 3 | 4 | 5 | (-1) | (-0.5) | 1.9 | 4.9 | "1" | TRUE'),
    # Coupon frequency: 1, 2 and 4, and what is #NUM! ("Analysis ToolPak securities functions check frequency and
    # dates first": 3, 0, 12, negatives, -0.0001), fractions (they truncate: 2.5 is 2 and 4.9 is 4) and number text
    # (coerced) ("Financial edge-pool smoke check (T-2248)").
    "frequency": entries('1 | 2 | 4 | 3 | 0 | 12 | 2.5 | 4.9 | (-1) | (-0.0001) | "2"'),
    # EFFECT and NOMINAL npery (floored, "EFFECT and NOMINAL raise to their power with ^"): 1 (an exact power), 2
    # (NOMINAL's square root), 4.5 (NOMINAL(0.5,4.5)), 12 and 365, values below 1 that floor to 0 (#NUM!), 0,
    # negatives and the large counts where (1+r/n)^n meets its limit.
    "npery": entries("1 | 2 | 4 | 4.5 | 12 | 365 | 0.5 | 0.999 | 1.5 | 0 | (-1) | 1E+6 | 1E+10"),
    # Cash flows of NPV, IRR, MIRR, XNPV, XIRR and FVSCHEDULE ("IRR, RATE and XIRR stop early", "MIRR error and edge
    # cases", "NPV, IRR and MIRR error order", "NPV and FVSCHEDULE arguments"): one sign change, none (#NUM! for IRR
    # and MIRR), several (more than one root), leading and trailing zeros, a single flow, all zeros, a long flat
    # run, huge flows, a column instead of a row, and text, a boolean and an error inside the array.
    "cash_flows": entries(
        "{-100,10,20,30,40} | {-1000,300,400,500} | {100,200,300} | {-100,-200,-300} | {-100,230,-132} | "
        "{-100,300,-300,101} | {0,0,-100,50,60} | {-100,50,60,0,0} | {-1,0,0,0,0,0,0,0,0,1000} | {-100,100} | "
        '{-100} | {0,0,0} | {-1E+300,1E+300} | {-100;30;40;50} | {-100,"a",60} | {-100,TRUE,60} | {-100,#N/A,60}'
    ),
    # XNPV and XIRR dates, five per entry like most cash_flows entries (other sizes are #NUM!): increasing,
    # all the same day, the first not the earliest (XIRR accepts earlier dates, "XIRR reads empty cells as 0 and
    # accepts earlier dates"), serials 0-61 around the fictitious 1900-02-29, a leap day and fractional serials.
    "cash_dates": entries(
        "{39448,39508,39859,39904,40000} | {39448,39448,39448,39448,39448} | {40000,39448,39508,39859,39904} | "
        "{0,1,59,60,61} | {45322,45351,45352,45657,45658} | {39448.5,39508.25,39859,39904,40000.9} | "
        "{39448,39508,39859} | {39448;39508;39859;39904;40000}"
    ),
    # The M9 engineering functions (docs/excel-semantics/engineering.md). IM* inumbers: a bare i or j, a zero
    # imaginary part, the j suffix and mixed suffixes ("T-1625 probes": IMSUB and IMDIV give #NUM!, IMSUM #VALUE!),
    # an upper-case I (COMPLEX's suffix is case-sensitive), E notation and parts near the double range (IM* results
    # that overflow: IMEXP("710"), IMPOWER("1E155",2)), malformed text (#NUM!), empty text (#NUM!, "Engineering edge
    # cases from a generated sample"), a number (read as its digits), a boolean (#VALUE!) and a blank cell (0;
    # Data!Z1 is never filled).
    "complex_text": entries(
        '"i" | "-i" | "j" | "3+0i" | "0" | "0i" | "3+4j" | "1+2I" | "1.5E+2-2E-3i" | "1E+200" | "1E-200+1E-200i" | '
        '"1E+300+1E+300i" | "710" | "1000+1i" | "0.99999973i" | "-724.227-562.884i" | "i3" | "3+4" | "3+4ii" | '
        '"+i" | "--1" | " 3+4i" | "3 + 4i" | "x" | "" | 3 | (-2.5) | TRUE | Data!Z1'
    ),
    # COMPLEX's suffix: i and j, the upper-case forms and other text (#VALUE!), empty text (the default i), a
    # number and a boolean.
    "complex_suffix": entries('"i" | "j" | "I" | "J" | "" | "k" | "ij" | " i" | 1 | TRUE'),
    # BIN2DEC, BIN2HEX and BIN2OCT numbers: 10 binary digits at most, the 10th the sign bit of a two's-complement
    # negative (1000000000 is -512, 1111111111 is -1); 11 digits, a digit 2, a fraction and a negative are #NUM!;
    # leading zeros, digits as text, empty text and a blank cell (0: "Base conversions read a blank cell and empty
    # text as 0"), a boolean (#VALUE!).
    "bin_digits": entries(
        '0 | 1 | 111111111 | 1000000000 | 1111111111 | 1111111110 | 10000000000 | 2 | 1.5 | (-1) | "0000000001" | '
        '"1111111111" | "" | Data!Z1 | TRUE'
    ),
    # OCT2BIN, OCT2DEC and OCT2HEX numbers: 3777777777 is the largest positive, 4000000000 the most negative and
    # 7777777777 is -1; 11 digits, an 8 or 9 digit and a fraction are #NUM!; OCT2BIN of a negative beyond -512
    # (7777777000) and of a positive past 511 (1000) has no 10-bit form; a blank cell is 0 ("OCT2BIN of a blank
    # cell is 0").
    "oct_digits": entries(
        "0 | 7 | 777 | 1000 | 3777777777 | 4000000000 | 7777777777 | 7777777000 | 77777777777 | 8 | 19 | 1.5 | (-1) | "
        '"0000000017" | "" | Data!Z1 | TRUE'
    ),
    # HEX2BIN, HEX2DEC and HEX2OCT numbers: 7FFFFFFFFF and 8000000000 (the two's-complement turn), FFFFFFFFFF (-1),
    # lower case, 11 digits and a non-hex letter (#NUM!), leading zeros, a number read as its digits (HEX2DEC(255)
    # is 597), HEX2BIN past its 10-bit range (200, FFFFFFFDFF), HEX2OCT past its 30-bit one (20000000), empty text.
    "hex_digits": entries(
        '"0" | "1F" | "1f" | "1FF" | "200" | "7FFFFFFFFF" | "8000000000" | "FFFFFFFFFF" | "FFFFFFFDFF" | "FFFFFFFE00" | '
        '"20000000" | "FFE0000000" | "00000000FF" | "10000000000" | "G" | " 1" | "-1" | "1.5" | 255 | "" | Data!Z1 | TRUE'
    ),
    # DEC2BIN, DEC2HEX and DEC2OCT numbers: the ends of each range (DEC2BIN -512..511, DEC2OCT -2^29..2^29-1,
    # DEC2HEX -2^39..2^39-1) and the first values past them, fractions (they truncate), number text and a boolean.
    "dec_number": entries(
        "0 | 1 | (-1) | 511 | 512 | (-512) | (-513) | 536870911 | 536870912 | (-536870912) | (-536870913) | "
        '549755813887 | 549755813888 | (-549755813888) | (-549755813889) | 1.9 | (-1.9) | 0.5 | "9" | TRUE | Data!Z1'
    ),
    # `places` of the base conversions: 1 to 10 work, 0, -1, 11 and "x" do not ("A negative base-conversion result
    # ignores places"), a fraction truncates, number text is read, and a positive result longer than `places` is
    # #NUM! (1 and 2 are below most results' length).
    "places": entries('1 | 2 | 3 | 9 | 10 | 11 | 0 | (-1) | 2.9 | 10.5 | "4" | "x" | TRUE'),
    # Bessel x ("BESSELJ / BESSELY (T-0900)"): 0 (BESSELY and BESSELK of x <= 0 are #NUM!), tiny values, both sides
    # of the x = 8 switch from the rational approximation to the asymptotic branch, 100 (the largest probed) and
    # beyond (unprobed, T-2140), BESSELI's overflow near 713, negatives (J and I take them) and number text.
    "bessel_x": entries(
        "0 | 1E-300 | 1E-10 | 0.5 | 1 | 2.5 | 7.999999 | 8 | 8.000001 | 10 | 30 | 100 | 1000 | 1E+5 | 700 | 713 | "
        '714 | (-1) | (-8.5) | (-1E-10) | "2.5" | TRUE'
    ),
    # Bessel order n: a positive fraction truncates (BESSELJ(8,9.99) is BESSELJ(8,9)), any negative is #NUM!
    # before truncation (-0.0001), orders above x (Miller's recurrence), 100000 (our cap, "BESSEL order cap") and
    # past it (Excel's BESSELJ(10,1000000) is 0; a DEVIATION), number text and a boolean.
    "bessel_order": entries(
        '0 | 1 | 2 | 0.9999 | 9.99 | 10 | 11 | 100 | 101 | 1000 | 100000 | 100001 | 1E+6 | (-0.0001) | (-1) | "1" | TRUE'
    ),
    # CONVERT units ("CONVERT constants and evaluation order", "CONVERT temperature"): base units, prefixed and
    # binary-prefixed ones, long names, units whose class keeps a non-SI base (dyn, erg, l, ang2), every
    # temperature scale with the prefixable K and kel and the unprefixable mC, kF, units of different classes
    # (#N/A), unknown and empty units, and case (units are case-sensitive).
    "convert_unit": entries(
        '"m" | "km" | "mm" | "Ym" | "ft" | "in" | "mi" | "Nmi" | "ang" | "parsec" | "ly" | "g" | "kg" | "lbm" | '
        '"sg" | "u" | "C" | "cel" | "F" | "K" | "kel" | "mK" | "kK" | "Rank" | "Reau" | "mC" | "kF" | "s" | "hr" | '
        '"day" | "yr" | "J" | "cal" | "eV" | "erg" | "N" | "dyn" | "lbf" | "m2" | "ha" | "ang2" | "m3" | "l" | '
        '"gal" | "uk_pt" | "tsp" | "tbs" | "Pa" | "atm" | "W" | "HP" | "T" | "ga" | "bit" | "byte" | "kibyte" | '
        '"Kibyte" | "M" | "KM" | "xyz" | "" | " m" | 1'
    ),
    # ERF, ERFC and their .PRECISE forms: 0 and tiny values, the point where ERF rounds to 1 (about 5.9) and
    # where ERFC underflows (about 27.2), negatives (ERF of a negative lower limit, ERFC near 2), huge values
    # and number text.
    "erf_x": entries(
        "0 | 1E-300 | (-1E-300) | 1E-10 | 0.5 | 1 | 2 | 3 | 5.9 | 6 | 10 | 26.5 | 27 | 27.3 | 28 | (-0.5) | "
        '(-1) | (-6) | (-27.3) | 1E+300 | (-1E+300) | "0.5" | TRUE'
    ),
    # BITAND, BITOR, BITXOR and the shifts: 2^48-1 is the largest number and 2^48 is #NUM! (numbers.md "CHISQ.DIST
    # degrees of freedom and BITAND operands are bounded"); a fraction and a negative are #NUM!, number text is
    # read, a boolean is its number (BITAND(TRUE,139) is 1) and a blank cell is 0 (engineering.md "Engineering
    # edge-pool smoke check (T-2249)").
    "bit_number": entries(
        "0 | 1 | 13 | 2^47 | 2^48-1 | 2^48 | 281474976710655 | 281474976710656 | 1.5 | 0.9999999999 | (-1) | "
        '"13" | TRUE | Data!Z1'
    ),
    # BITLSHIFT and BITRSHIFT shift_amount: 48 and 54 are #NUM! for BITLSHIFT(1,..) but 53 and -53 give 0
    # (T-2421); a negative shifts the other way and a fraction truncates (the T-2249 smoke check).
    "bit_shift": entries('0 | 1 | (-1) | 47 | 48 | 53 | 54 | (-53) | (-54) | 1.5 | (-1.5) | "2" | TRUE'),
    # The D* database (gen/database.py's sheet Db): the whole table, absolute, the header row alone (no
    # records), one column, the first four records, trailing blank records, the table without its Name column,
    # and the table without its header row (the first record is read as headers).
    "db_table": entries(
        "Db!A1:D9 | Db!$A$1:$D$9 | Db!A1:D1 | Db!A1:B9 | Db!A1:D5 | Db!A1:D12 | Db!B1:D9 | Db!A2:D9"
    ),
    # The D* field ("Database function argument modes"): column indexes 1-4, past the table (5) and 0, a negative
    # and a fraction, headers by name in either case and with a space, an unknown name and numeric text
    # (#VALUE!), a boolean (no column: 0, #DIV/0! or #NUM!), empty text and a header cell reference.
    "db_field": entries(
        '1 | 2 | 3 | 4 | 5 | 0 | (-1) | 2.9 | "Qty" | "qty" | "Price" | " Qty" | "Name" | "Nope" | "2" | TRUE | "" | '
        "Db!B1 | Db!C1"
    ),
    # The D* criteria (gen/database.py's blocks): each block alone, two adjacent blocks (an AND across columns),
    # a block with a blank row under it (a blank criteria row matches every record), the header row only, a
    # header-only cell, absolute references and a block whose header the table does not have.
    "db_criteria": entries(
        "Db!F1:F2 | Db!G1:G2 | Db!H1:H2 | Db!I1:I2 | Db!J1:J2 | Db!K1:K2 | Db!L1:L3 | Db!M1:M2 | Db!N1:N2 | "
        "Db!O1:O2 | Db!P1:P2 | Db!Q1:Q2 | Db!R1:R2 | Db!S1:S2 | Db!F1:G2 | Db!G1:H2 | Db!F1:F3 | Db!F1:G1 | "
        "Db!F1 | Db!$G$1:$G$2 | Db!F1:Q2"
    ),
}


def lambda_arity(text: str) -> int:
    """The parameter count of a `lambda_body` entry: `LAMBDA(x,[y],ISOMITTED(y))` has 2."""
    match = re.match(r"LAMBDA\(((?:\[?[a-z]+\]?,)*)", text)
    return match.group(1).count(",") if match else 0
