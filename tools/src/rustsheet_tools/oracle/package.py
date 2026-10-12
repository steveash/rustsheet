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
from xml.etree import ElementTree

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


def _relationships(part: str, data: bytes) -> list[dict[str, str]]:
    """The attributes of each Relationship in a rels part; a rels part that does not parse is an error."""
    # A relationships part never needs a DTD; refusing one rules out entity-expansion bombs.
    if b"<!DOCTYPE" in data.upper() or b"<!ENTITY" in data.upper():
        raise PackageError(f"unreadable relationships in {part} (it declares a DTD)")
    try:
        root = ElementTree.fromstring(data)  # noqa: S314 - no DTD, checked above
    except ElementTree.ParseError as e:
        raise PackageError(f"unreadable relationships in {part} ({e})") from e
    return [dict(el.attrib) for el in root.iter() if el.tag.rpartition("}")[2] == "Relationship"]


def _resolve(base_dir: str, target: str) -> str:
    """A relationship target as a part name (absolute targets start at the package root)."""
    return target[1:] if target.startswith("/") else posixpath.normpath(posixpath.join(base_dir, target))


def workbook_part(parts: dict[str, bytes]) -> str:
    """Name of the workbook part, from the package's root relationships."""
    if "_rels/.rels" not in parts:
        raise PackageError("unreadable xlsx (no _rels/.rels)")
    for rel in _relationships("_rels/.rels", parts["_rels/.rels"]):
        if rel.get("Type", "").endswith("/officeDocument") and (target := rel.get("Target")):
            name = _resolve("", target)
            if name in parts:
                return name
    raise PackageError("workbook part not found")


def rels_part(part: str) -> str:
    """Name of the relationships part that belongs to `part`."""
    directory, file = posixpath.split(part)
    return posixpath.join(directory, "_rels", f"{file}.rels")


def formula_parts(parts: dict[str, bytes], workbook: str) -> list[str]:
    """The workbook part plus every part that can hold formulas, for a refusal scan.

    That is every sheet part the workbook's relationships point at, wherever it is stored, plus
    everything under the workbook's `worksheets/` directory (the usual home, kept as a second net).
    Fails closed: a relationship to a sheet part that is not in the package is an error.
    """
    base = posixpath.dirname(workbook)
    names = [workbook]
    if (rels := parts.get(rels_part(workbook))) is not None:
        for rel in _relationships(rels_part(workbook), rels):
            if not _FORMULA_SHEET_TYPE.search(rel.get("Type", "")) or rel.get("TargetMode") == "External":
                continue
            name = _resolve(base, rel.get("Target", ""))
            if name not in parts:
                raise PackageError(f"{rels_part(workbook)} points at {name}, which is not in the package")
            names.append(name)
    prefix = posixpath.join(base, "worksheets/")
    names.extend(n for n in parts if n.startswith(prefix))
    return list(dict.fromkeys(names))


def patch_workbook(xlsx: bytes, patch: Callable[[str], str]) -> bytes:
    """`xlsx` with the workbook part's text replaced by `patch(text)`; every other part is untouched."""
    parts = read_parts(xlsx)
    name = workbook_part(parts)
    try:
        parts[name] = patch(parts[name].decode("utf-8")).encode("utf-8")
    except UnicodeDecodeError as e:
        raise PackageError(f"unreadable xlsx ({e})") from e
    return write_parts(parts)
