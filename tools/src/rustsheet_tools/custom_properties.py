"""JSON values kept in an xlsx's docProps custom properties (spec2xlsx records oracle-backend steps there).

Excel caps a property value, so a long one is split over continuation properties (T-2709).
"""

from __future__ import annotations

import json
import re
from typing import Any
from xml.sax.saxutils import unescape

import xlsxwriter

from rustsheet_tools.oracle.package import read_parts

# Excel holds at most 255 characters (UTF-16 units) in a custom property, so a longer JSON value is
# split over `<name>` and continuation properties `<name>.2`, `<name>.3`, ... (T-2709).
_PROPERTY_UNITS = 255
# The longest JSON value recorded: an entered formula holds at most Excel's 8,192 characters, and a
# few hundred more cover the sheet name, the address and JSON escaping. Larger fails here, not at
# the oracle.
_PROPERTY_JSON_MAX = 16_384


def _utf16_chunks(text: str, units: int) -> list[str]:
    """`text` cut into pieces of at most `units` UTF-16 units, never inside a surrogate pair."""
    chunks: list[str] = []
    start, used = 0, 0
    for i, ch in enumerate(text):
        width = 2 if ord(ch) > 0xFFFF else 1
        if used + width > units:
            chunks.append(text[start:i])
            start, used = i, 0
        used += width
    chunks.append(text[start:])
    return chunks


def set_json_property(wb: xlsxwriter.Workbook, name: str, value: object, what: str) -> None:
    """Records `value` as JSON in the custom property `name`, continued in `name.2`, ... past 255 units."""
    text = json.dumps(value, ensure_ascii=False)
    if len(text) > _PROPERTY_JSON_MAX:
        msg = f"{what} is too long for the custom properties ({len(text)} > {_PROPERTY_JSON_MAX})"
        raise ValueError(msg)
    first, *rest = _utf16_chunks(text, _PROPERTY_UNITS)
    wb.set_custom_property(name, first)
    for k, chunk in enumerate(rest, start=2):
        wb.set_custom_property(f"{name}.{k}", chunk)


def json_properties(xlsx: bytes, prefix: str) -> list[Any]:
    """The JSON values of the custom properties `<prefix>.<n>` in `xlsx`, in order of n.

    Raises ValueError when the package or a property is malformed (xlsx inputs are untrusted).
    """
    try:
        parts = read_parts(xlsx)
        if "docProps/custom.xml" not in parts:
            return []
        custom = parts["docProps/custom.xml"].decode("utf-8")
        # A regex, not an XML parser: the part is untrusted, and only XlsxWriter's own layout matters.
        # `<prefix>.<n>` is value n, or its first piece when `<prefix>.<n>.<k>` continue it (k >= 2).
        pattern = re.compile(
            rf'name="{re.escape(prefix)}\.(\d+)(?:\.(\d+))?"><vt:lpwstr>(.*?)</vt:lpwstr>', re.S
        )
        pieces: dict[int, dict[int, str]] = {}
        for n, k, v in pattern.findall(custom):
            pieces.setdefault(int(n), {})[int(k) if k else 1] = unescape(v, {"&quot;": '"', "&apos;": "'"})
        values = []
        for n in sorted(pieces):
            parts_of_n = pieces[n]
            if sorted(parts_of_n) != list(range(1, len(parts_of_n) + 1)):
                msg = f"{prefix}.{n} is missing a piece"
                raise ValueError(msg)
            values.append(json.loads("".join(parts_of_n[k] for k in sorted(parts_of_n))))
        return values
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as e:
        msg = f"malformed {prefix} property: {e}"
        raise ValueError(msg) from e
