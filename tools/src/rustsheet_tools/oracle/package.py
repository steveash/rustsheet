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
from collections.abc import Callable, Iterator
from xml.etree import ElementTree
from xml.parsers import expat

# Part types that carry formulas: worksheets, XLM macro sheets, dialog sheets. Chartsheets hold none.
_FORMULA_SHEET_TYPE = re.compile(r"/(?:worksheet|xlMacrosheet|dialogsheet)$")


# Caps on what the tooling decompresses (directive 5: workbooks are untrusted). Oracle workbooks are
# a few parts and well under a megabyte; the caps leave orders of magnitude of headroom.
MAX_PARTS = 10_000
MAX_UNCOMPRESSED_BYTES = 256 * 1024 * 1024


class PackageError(ValueError):
    """The bytes are not an xlsx package this tooling can read or rewrite."""


def _check_size(infos: list[zipfile.ZipInfo]) -> None:
    """Refuses too many entries or too many uncompressed bytes, from the directory, before any read."""
    if len(infos) > MAX_PARTS:
        raise PackageError(f"xlsx has too many parts ({len(infos)} > {MAX_PARTS})")
    if sum(i.file_size for i in infos) > MAX_UNCOMPRESSED_BYTES:
        raise PackageError(f"xlsx is over {MAX_UNCOMPRESSED_BYTES} bytes uncompressed")


def read_parts(xlsx: bytes) -> dict[str, bytes]:
    """Every part of the package by name, in archive order."""
    try:
        with zipfile.ZipFile(io.BytesIO(xlsx)) as zf:
            infos = zf.infolist()
            _check_size(infos)
            # Readers disagree on which of two same-named entries wins, so a scan could pass one
            # copy while Excel loads the other.
            if len({i.filename for i in infos}) != len(infos):
                raise PackageError("unreadable xlsx (duplicate part names)")
            return {i.filename: zf.read(i) for i in infos}
    except zipfile.BadZipFile as e:
        raise PackageError(f"unreadable xlsx ({e})") from e


def write_parts(parts: dict[str, bytes]) -> bytes:
    """A package holding exactly `parts`."""
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, data in parts.items():
            zf.writestr(name, data)
    return out.getvalue()


def patch_parts(xlsx: bytes, patches: list[tuple[str, Callable[[str], str]]]) -> bytes:
    """`xlsx` with each `(part name, patch)` applied to that part's text, in order.

    Keeps every entry's `ZipInfo` (timestamp, compression), so unpatched parts and the archive
    layout are unchanged (unlike `write_parts`, which restamps every entry, so
    callers that need byte-stable output use this). A part that is not in the package is an error.
    """
    by_part: dict[str, list[Callable[[str], str]]] = {}
    for part, patch in patches:
        by_part.setdefault(part, []).append(patch)
    out = io.BytesIO()
    try:
        with zipfile.ZipFile(io.BytesIO(xlsx)) as src, zipfile.ZipFile(out, "w") as dst:
            _check_size(src.infolist())
            for info in src.infolist():
                data = src.read(info.filename)
                if info.filename in by_part:
                    text = data.decode()
                    for patch in by_part[info.filename]:
                        text = patch(text)
                    data = text.encode()
                dst.writestr(info, data)
            if missing := by_part.keys() - set(src.namelist()):
                raise PackageError(f"parts not in the package: {sorted(missing)}")
    except zipfile.BadZipFile as e:
        raise PackageError(f"unreadable xlsx ({e})") from e
    return out.getvalue()


def _refuse_dtd(*_: object) -> None:
    raise PackageError("it declares a DTD")


def parse_part(part: str, data: bytes) -> ElementTree.Element:
    """The XML tree of a part; a part that does not parse, or declares a DTD, is an error.

    No package part needs a DTD, so refusing one rules out entity-expansion bombs and external
    entities. Expat reads the encoding itself (UTF-8, UTF-16 with a BOM, an `encoding=`
    declaration), so a DTD cannot hide behind an encoding the way it could from a byte search.
    """
    check = expat.ParserCreate()
    check.StartDoctypeDeclHandler = _refuse_dtd
    check.EntityDeclHandler = _refuse_dtd
    try:
        check.Parse(data, True)
        # The checking pass proved there is no DTD, so the tree parse cannot expand entities.
        return ElementTree.fromstring(data)  # noqa: S314 - DTD refused by the pass above
    except (expat.ExpatError, ElementTree.ParseError, ValueError) as e:  # ValueError: _refuse_dtd
        raise PackageError(f"unreadable {part} ({e})") from e


def _local(tag: str) -> str:
    return tag.rpartition("}")[2]


def _relationships(part: str, data: bytes) -> list[dict[str, str]]:
    """The attributes of each Relationship in a rels part."""
    root = parse_part(part, data)
    return [dict(el.attrib) for el in root.iter() if _local(el.tag) == "Relationship"]


def text_runs(part: str, data: bytes) -> Iterator[str]:
    """Every run of character data and every attribute value in a part, unescaped.

    For pattern scans: entity and character references are resolved and comments and CDATA
    sections do not split a run, so `SHEETS&#40;)` or `SHE<!---->ETS()` read as `SHEETS()`.
    """
    for el in parse_part(part, data).iter():
        yield from el.attrib.values()
        if el.text:
            yield el.text
        if el.tail:
            yield el.tail


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
