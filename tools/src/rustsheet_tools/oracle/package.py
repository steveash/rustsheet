"""The one place oracle tooling reads and rewrites an xlsx package (a zip of XML parts).

Callers patch part *text* (a flag, a sheet entry); values are still decoded only by the Rust
snapshot (decision 0006). The workbook part is found through `_rels/.rels`, never assumed to be
`xl/workbook.xml`, and worksheets through the workbook's own relationships, never by directory.
Problems raise `PackageError`; backends and the canary translate it into `OracleError`.
"""

from __future__ import annotations

import io
import posixpath
import re
import zipfile
from collections.abc import Callable

_RELATIONSHIP = re.compile(r"<(?:\w+:)?Relationship\b[^>]*>")
# Part types that carry formulas: worksheets, XLM macro sheets, dialog sheets. Chartsheets hold none.
_FORMULA_SHEET_TYPE = re.compile(r"/(?:worksheet|xlMacrosheet|dialogsheet)$")


class PackageError(ValueError):
    """The bytes are not an xlsx package this tooling can read or rewrite."""


def read_parts(xlsx: bytes) -> dict[str, bytes]:
    """Every part of the package by name, in archive order."""
    try:
        with zipfile.ZipFile(io.BytesIO(xlsx)) as zf:
            return {i.filename: zf.read(i.filename) for i in zf.infolist()}
    except zipfile.BadZipFile as e:
        raise PackageError(f"unreadable xlsx ({e})") from e


def write_parts(parts: dict[str, bytes]) -> bytes:
    """A package holding exactly `parts`."""
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, data in parts.items():
            zf.writestr(name, data)
    return out.getvalue()


def _attr(tag: str, name: str) -> str | None:
    m = re.search(rf'\b{name}="([^"]*)"', tag)
    return m.group(1) if m else None


def _resolve(base_dir: str, target: str) -> str:
    """A relationship target as a part name (absolute targets start at the package root)."""
    return target[1:] if target.startswith("/") else posixpath.normpath(posixpath.join(base_dir, target))


def workbook_part(parts: dict[str, bytes]) -> str:
    """Name of the workbook part, from the package's root relationships."""
    try:
        root_rels = parts["_rels/.rels"].decode("utf-8")
    except (KeyError, UnicodeDecodeError) as e:
        raise PackageError(f"unreadable xlsx ({e})") from e
    for tag in _RELATIONSHIP.findall(root_rels):
        if (_attr(tag, "Type") or "").endswith("/officeDocument") and (target := _attr(tag, "Target")):
            name = _resolve("", target)
            if name in parts:
                return name
    raise PackageError("workbook part not found")


def rels_part(part: str) -> str:
    """Name of the relationships part that belongs to `part`."""
    directory, file = posixpath.split(part)
    return posixpath.join(directory, "_rels", f"{file}.rels")


def formula_parts(parts: dict[str, bytes], workbook: str) -> list[str]:
    """The workbook part plus every sheet part its relationships point at, wherever they are stored."""
    rels = parts.get(rels_part(workbook))
    names = [workbook]
    if rels is None:
        return names
    base = posixpath.dirname(workbook)
    for tag in _RELATIONSHIP.findall(rels.decode("utf-8", "replace")):
        target = _attr(tag, "Target")
        if (
            target
            and _FORMULA_SHEET_TYPE.search(_attr(tag, "Type") or "")
            and _attr(tag, "TargetMode") != "External"
        ):
            name = _resolve(base, target)
            if name in parts and name not in names:
                names.append(name)
    return names


def patch_workbook(xlsx: bytes, patch: Callable[[str], str]) -> bytes:
    """`xlsx` with the workbook part's text replaced by `patch(text)`; every other part is untouched."""
    parts = read_parts(xlsx)
    name = workbook_part(parts)
    try:
        parts[name] = patch(parts[name].decode("utf-8")).encode("utf-8")
    except UnicodeDecodeError as e:
        raise PackageError(f"unreadable xlsx ({e})") from e
    return write_parts(parts)
