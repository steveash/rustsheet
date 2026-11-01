"""T-2681: the arguments of gen-m9/bcb24385baafb2f0 Calc!A377,
`FTEST(Data!B7:E9,FILTER(Data!A1:D12,(Data!D1:D12>7)))`, which the fit sweep leaves unparsed.

FILTER keeps the rows whose D cell compares above 7: a number above 7, and text or TRUE (Excel
orders numbers < text < logicals), so rows 1 (TRUE), 6 (42), 8 ("1.5") and 10 (1E15); F.TEST
then skips the text and logical elements. Run from tools/: `uv run python
../docs/research/t2681/filter_cell.py`; it prints (x, y, a, b), the model's p and Excel's.
"""

from rustsheet_tools import repo_root
from rustsheet_tools.fitlab.kernels import ftest as F
from rustsheet_tools.fitlab.sweep import Cell, load_workbook, parse_arg

cases = repo_root() / "eval" / "suites" / "gen-m9" / "cases"
book = load_workbook(cases / "bcb24385baafb2f0.case.json", cases / "bcb24385baafb2f0.oracle.json")
first = F._numbers(parse_arg("Data!B7:E9", "Calc", book))
kept = []
for row in range(1, 13):
    d = book.value("Data", f"D{row}")
    if isinstance(d, str | bool) or (isinstance(d, float) and d > 7):
        kept += [book.value("Data", f"{col}{row}") for col in "ABCD"]
second = F._numbers(kept)
print(F._shape(Cell("", "", "", "", (first, second), 0.0)), F.variant()(first, second), book.oracle["Calc"]["A377"])
