"""Slicers and timelines in spec cases (T-4767), so a probe can ask whether Excel for the web opens a file with
them and whether their selection filters the pivot (docs/research/pivot-tables-2026-11-04/plan.md §7).

A sheet's `"slicers"` and `"timelines"` list them, each bound to a pivot of the workbook (a slicer may name a
table column instead):

    "slicers": [{"pivot": "P_S", "field": "Region", "anchor": "E3",
                 "selected": ["East"],                         # its items with s="1" (default: every item)
                 "item_attrs": {"West": {"nd": True}},         # other `<i>` attributes
                 "tabular": {"sortOrder": "descending"},       # `<tabular>` attributes
                 "slicer": {"columnCount": 2},                 # `<slicer>` attributes
                 "cache_ext": "<extLst>...</extLst>",          # the slicer cache's raw extLst
                 "pivot_cache": {"slicerData": False}},        # x14:pivotCacheDefinition attributes
                {"table": "T1", "column": "Region", "anchor": "E3", "table_slicer": {"sortOrder": "descending"}}]
    "timelines": [{"pivot": "P_T", "field": "Date", "anchor": "E3",
                   "bounds": [43466, 44197], "selection": [43466, 43555],   # date serials (selection optional)
                   "state": {"singleRangeFilterState": False},              # `<state>` attributes
                   "timeline": {"level": 2},                                # `<timeline>` attributes
                   "pivot_filter": "<timelinePivotFilter>...</timelinePivotFilter>",  # raw
                   "timeline_pivot_cache": {"timelineData": False}}]        # x15:timelinePivotCacheDefinition

The parts are the ones Excel writes ([MS-XLSX] 2.2.x; the extension uris and part types as Excel-saved files
carry them, e.g. EPPlus's Control.xlsm for a table slicer): a slicer cache or timeline cache part per entry,
listed in the workbook's extLst (`x14:slicerCaches`, `x15:timelineCacheRefs`, `x15:slicerCaches` for a table
slicer) with a `#N/A` defined name; a slicers or timelines part per sheet, listed in the sheet's extLst; a drawing
frame per entry (`sle:slicer`, `tsle:timeslicer`); and the pivot cache's `x14:pivotCacheDefinition
pivotCacheId`, which the slicer's `tabular` and the timeline's `state` name. The pivot itself is left as the
definition writes it: whether Excel applies the selection is the probe's question. The engine adapter ignores
the keys (decision 0274: slicers are T-4188's), as it does `pivot_charts`.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any
from xml.sax.saxutils import quoteattr

import xlsxwriter.utility

from rustsheet_tools.oracle.package import patch_parts, read_parts
from rustsheet_tools.pivot import relationships_xml, xml_part
from rustsheet_tools.pivot_chart import _append, _insert_drawing
from rustsheet_tools.pivot_definition import _attrs, raw_element
from rustsheet_tools.pivot_xml import MAIN_NS, REL_NS, iso_date

X14_NS = "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"
X15_NS = "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main"
_MC_NS = "http://schemas.openxmlformats.org/markup-compatibility/2006"
_A_NS = "http://schemas.openxmlformats.org/drawingml/2006/main"
_XDR_NS = "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing"
_A14_NS = "http://schemas.microsoft.com/office/drawing/2010/main"
_SLE_NS = "http://schemas.microsoft.com/office/drawing/2010/slicer"
_SLE15_NS = "http://schemas.microsoft.com/office/drawing/2012/slicer"
_TSLE_NS = "http://schemas.microsoft.com/office/drawing/2012/timeslicer"
_MS_REL = "http://schemas.microsoft.com/office"
# Extension uris: the workbook's, the sheet's, the slicer cache's and the pivot cache's.
_WB_SLICER_CACHES = "{BBE1A952-AA13-448e-AADC-164F8A28A991}"
_WB_TIMELINE_CACHES = "{D0CA8CA8-9F24-4464-BF8E-62219DCF47F9}"
_WB_TABLE_SLICER_CACHES = "{46BE6895-7355-4a93-B00E-2C351335B9C9}"
_SHEET_SLICERS = "{A8765BA9-456A-4dab-B4F3-ACF838C121DE}"
_SHEET_TABLE_SLICERS = "{3A4CF648-6AED-40f4-86FF-DC5316D8AED3}"
_SHEET_TIMELINES = "{7E03D99C-DC04-49d9-9315-930204A7B6E9}"
_TABLE_SLICER_CACHE = "{2F2917AC-EB37-4324-AD4E-5DD8C200BD13}"
_PIVOT_CACHE_X14 = "{725AE2AE-9491-48be-B2B4-4EB974FC3084}"
_PIVOT_CACHE_TIMELINE = "{5DA0FC9A-693D-419c-AD59-312A39285967}"
_DRAWING_TYPE = "application/vnd.openxmlformats-officedocument.drawing+xml"
_SLICER_KEYS = {"pivot", "table", "field", "column", "anchor", "name", "caption", "selected", "item_attrs",
                "tabular", "slicer", "cache_ext", "pivot_cache", "table_slicer"}  # fmt: skip
_TIMELINE_KEYS = {"pivot", "field", "anchor", "name", "caption", "bounds", "selection", "state", "timeline",
                  "pivot_filter", "timeline_pivot_cache"}  # fmt: skip
_ROOT = f'xmlns:mc="{_MC_NS}" mc:Ignorable="x" xmlns:x="{MAIN_NS}"'
_ITEM = re.compile(r'<(s|n|b|e|d|m)(?: v="([^"]*)")?/>')
_SELECTED = ' s="1"'
_ROW_HEIGHT = 241300  # Excel's default slicer button height, EMU


@dataclass
class _Pivot:
    """What a slicer or timeline needs of its pivot: where it is and its cache's fields."""

    sheet: str
    tab_id: int
    cache_id: int
    cache_part: str
    fields: dict[str, list[str]] = field(default_factory=dict)  # name -> shared item texts (empty: inline)


def _pivots(parts: dict[str, bytes], spec: dict[str, Any]) -> dict[str, _Pivot]:
    """Each pivot of the package by name, read off the parts the definition builder wrote."""
    workbook = parts["xl/workbook.xml"].decode()
    rels = parts["xl/_rels/workbook.xml.rels"].decode()
    targets = dict(re.findall(r'<Relationship Id="([^"]+)" Type="[^"]+" Target="([^"]+)"', rels))
    caches = {
        int(c): "xl/" + targets[r]
        for c, r in re.findall(r'<pivotCache cacheId="(\d+)" r:id="([^"]+)"', workbook)
    }
    tabs = {n: int(i) for n, i in re.findall(r'<sheet name="([^"]+)" sheetId="(\d+)"', workbook)}
    sheets = {str(p["name"]): str(s["name"]) for s in spec["sheets"] for p in s.get("pivot_tables", ())}
    out: dict[str, _Pivot] = {}
    for part, data in parts.items():
        if not re.fullmatch(r"xl/pivotTables/pivotTable\d+\.xml", part):
            continue
        head = re.search(r'<pivotTableDefinition [^>]*?name="([^"]+)" cacheId="(\d+)"', data.decode())
        if head is None or head[1] not in sheets:
            continue
        cache_id = int(head[2])
        sheet = sheets[head[1]]
        pivot = _Pivot(sheet, tabs[sheet.replace("&", "&amp;")], cache_id, caches[cache_id])
        for name, body in re.findall(r'<cacheField name="([^"]+)"[^>]*?(?:/>|>(.*?)</cacheField>)',
                                     parts[pivot.cache_part].decode()):  # fmt: skip
            listed = re.search(r"<sharedItems[^>]*>(.*?)</sharedItems>", body)
            pivot.fields[name] = [m[2] or "" for m in _ITEM.finditer(listed[1])] if listed else []
        out[head[1]] = pivot
    return out


def _date(raw: Any, where: str) -> str:
    if isinstance(raw, bool) or not isinstance(raw, int | float):
        raise ValueError(f"{where} is a date serial")
    return iso_date(float(raw))


@dataclass
class _Package:
    """The parts and patches the entries add, and the counters that name them."""

    pivots: dict[str, _Pivot]
    tables: dict[str, tuple[int, list[str]]]  # table name -> (id, column names)
    added: dict[str, bytes] = field(default_factory=dict)
    types: list[str] = field(default_factory=list)
    workbook_rels: list[str] = field(default_factory=list)
    names: list[str] = field(default_factory=list)
    cache_ext: dict[str, dict[str, str]] = field(default_factory=dict)  # cache part -> uri -> element
    cache_refs: list[tuple[str, str]] = field(default_factory=list)  # (workbook extension uri, rid)
    caches: int = 0
    used: set[str] = field(default_factory=set)  # cache and shape names given so far, folded

    def unique(self, base: str, sep: str) -> str:
        """`base`, or `base` with the first free number after `sep`, as Excel names a second slicer or
        timeline on one field (`Slicer_Region1`, shape `Region 1`): a workbook whose names repeat is refused."""
        name, k = base, 0
        while name.lower() in self.used:
            k += 1
            name = f"{base}{sep}{k}"
        self.used.add(name.lower())
        return name

    def add_cache(self, kind: str, folder: str, xml: str, defined_name: str, uri: str) -> None:
        self.caches += 1
        part = f"xl/{folder}/{kind}{self.caches}.xml"
        self.added[part] = xml_part(xml)
        mime = "slicerCache" if kind == "slicerCache" else "timelineCache"
        self.types.append(f'<Override PartName="/{part}" ContentType="application/vnd.ms-excel.{mime}+xml"/>')
        rel_year = "2007" if kind == "slicerCache" else "2011"
        rid = f"rIdSlicerCache{self.caches}"
        self.workbook_rels.append(
            f'<Relationship Id="{rid}" Type="{_MS_REL}/{rel_year}/relationships/{kind}" Target="{folder}/{kind}{self.caches}.xml"/>'
        )
        self.names.append(f"<definedName name={quoteattr(defined_name)}>#N/A</definedName>")
        self.cache_refs.append((uri, rid))

    def pivot_cache_ext(self, pivot: _Pivot, uri: str, element: str) -> None:
        self.cache_ext.setdefault(pivot.cache_part, {})[uri] = element


def _slicer(entry: Any, pkg: _Package, sheet: str) -> tuple[str, str, bool]:
    """One slicer's cache part; returns its `<slicer>` element, its shape name and whether it is a table's."""
    where = f"sheet {sheet}: slicer"
    if not isinstance(entry, dict) or (unknown := set(entry) - _SLICER_KEYS):
        raise ValueError(
            f"{where} is an object with keys {sorted(_SLICER_KEYS)}, not {sorted(unknown or [])}"
        )
    table = "table" in entry
    if table == ("pivot" in entry) or "anchor" not in entry:
        raise ValueError(f"{where} names a pivot or a table, and an anchor")
    source = str(entry.get("column" if table else "field", ""))
    name = pkg.unique(str(entry.get("name", f"Slicer_{source}")), "")
    caption = str(entry.get("caption", source))
    head = f'<slicerCacheDefinition xmlns="{X14_NS}" {_ROOT} name={quoteattr(name)} sourceName={quoteattr(source)}>'
    ext = raw_element(entry["cache_ext"], "extLst", f"{where} cache_ext") if "cache_ext" in entry else ""
    if table:
        if str(entry["table"]) not in pkg.tables or source not in pkg.tables[str(entry["table"])][1]:
            raise ValueError(f"{where}: table {entry['table']!r} has no column {source!r}")
        table_id, columns = pkg.tables[str(entry["table"])]
        attrs = _attrs(entry.get("table_slicer"), {"tableId", "column"}, f"{where} table_slicer")
        table_ext = (
            f'<x:ext uri="{_TABLE_SLICER_CACHE}" xmlns:x15="{X15_NS}"><x15:tableSlicerCache tableId="{table_id}"'
            f' column="{columns.index(source) + 1}"{attrs}/></x:ext>'
        )
        ext = f"<extLst>{table_ext}{ext[len('<extLst>') : -len('</extLst>')] if ext else ''}</extLst>"
        xml = f"{head}{ext}</slicerCacheDefinition>"
    else:
        xml = f"{head}{_tabular(entry, pkg, where, source)}{ext}</slicerCacheDefinition>"
    pkg.add_cache(
        "slicerCache", "slicerCaches", xml, name, _WB_TABLE_SLICER_CACHES if table else _WB_SLICER_CACHES
    )
    attrs = _attrs(entry.get("slicer"), {"name", "cache", "caption"}, f"{where} slicer")
    if "rowHeight" not in attrs:
        attrs += f' rowHeight="{_ROW_HEIGHT}"'
    shape = pkg.unique(caption, " ")
    return (
        f"<slicer name={quoteattr(shape)} cache={quoteattr(name)} caption={quoteattr(caption)}{attrs}/>",
        shape,
        table,
    )


def _tabular(entry: dict[str, Any], pkg: _Package, where: str, source: str) -> str:
    """A pivot slicer's `<pivotTables>` and `<data>`: every shared item of its field, selected or not."""
    pivot = pkg.pivots.get(str(entry["pivot"]))
    if pivot is None or not pivot.fields.get(source):
        raise ValueError(f"{where}: pivot {entry['pivot']!r} has no listed field {source!r}")
    items = pivot.fields[source]
    selected = entry.get("selected", items)
    item_attrs = entry.get("item_attrs") or {}
    if not isinstance(selected, list) or not isinstance(item_attrs, dict):
        raise ValueError(f"{where}: selected is a list of items and item_attrs an object")
    if missing := [str(v) for v in [*selected, *item_attrs] if str(v) not in items]:
        raise ValueError(f"{where}: items {missing} are not in field {source!r}")
    chosen = {str(v) for v in selected}
    listed = "".join(
        f'<i x="{k}"{_SELECTED if v in chosen else ""}'
        f"{_attrs(item_attrs.get(v), {'x', 's'}, f'{where} item {v!r}')}/>"
        for k, v in enumerate(items)
    )
    pkg.pivot_cache_ext(pivot, _PIVOT_CACHE_X14, _x14_cache(pivot, entry.get("pivot_cache"), where))
    tabular = _attrs(entry.get("tabular"), {"pivotCacheId"}, f"{where} tabular")
    return (
        f'<pivotTables><pivotTable tabId="{pivot.tab_id}" name={quoteattr(str(entry["pivot"]))}/></pivotTables>'
        f'<data><tabular pivotCacheId="{pivot.cache_id}"{tabular}><items count="{len(items)}">{listed}</items>'
        "</tabular></data>"
    )


def _x14_cache(pivot: _Pivot, raw: Any, where: str) -> str:
    attrs = _attrs(raw, {"pivotCacheId"}, f"{where} pivot_cache")
    return (
        f'<ext uri="{_PIVOT_CACHE_X14}" xmlns:x14="{X14_NS}"><x14:pivotCacheDefinition{attrs}'
        f' pivotCacheId="{pivot.cache_id}"/></ext>'
    )


def _timeline(entry: Any, pkg: _Package, sheet: str) -> tuple[str, str]:
    """One timeline's cache part; returns its `<timeline>` element and its shape name."""
    where = f"sheet {sheet}: timeline"
    if not isinstance(entry, dict) or (unknown := set(entry) - _TIMELINE_KEYS):
        raise ValueError(
            f"{where} is an object with keys {sorted(_TIMELINE_KEYS)}, not {sorted(unknown or [])}"
        )
    pivot = pkg.pivots.get(str(entry.get("pivot")))
    source = str(entry.get("field", ""))
    if pivot is None or source not in pivot.fields or "anchor" not in entry:
        raise ValueError(f"{where}: names a pivot of the workbook, one of its fields and an anchor")
    bounds, selection = entry.get("bounds"), entry.get("selection")
    if (
        not isinstance(bounds, list)
        or len(bounds) != 2
        or not (selection is None or (isinstance(selection, list) and len(selection) == 2))
    ):
        raise ValueError(f"{where}: bounds (and selection) are [start, end] date serials")
    name = pkg.unique(str(entry.get("name", f"NativeTimeline_{source}")), "")
    caption = str(entry.get("caption", source))
    chosen = (
        f'<selection startDate="{_date(selection[0], where)}" endDate="{_date(selection[1], where)}"/>'
        if selection
        else ""
    )
    state = _attrs(entry.get("state"), {"pivotCacheId"}, f"{where} state")
    if "filterType" not in state:
        state += f' filterType="{"dateBetween" if selection else "unknown"}"'
    pivot_filter = (
        raw_element(entry["pivot_filter"], "timelinePivotFilter", f"{where} pivot_filter")
        if "pivot_filter" in entry
        else ""
    )
    xml = (
        f'<timelineCacheDefinition xmlns="{X15_NS}" {_ROOT} name={quoteattr(name)} sourceName={quoteattr(source)}>'
        f'<pivotTables><pivotTable tabId="{pivot.tab_id}" name={quoteattr(str(entry["pivot"]))}/></pivotTables>'
        f'<state minimalRefreshVersion="6" lastRefreshVersion="6" pivotCacheId="{pivot.cache_id}"{state}>{chosen}'
        f'<bounds startDate="{_date(bounds[0], where)}" endDate="{_date(bounds[1], where)}"/></state>'
        f"{pivot_filter}</timelineCacheDefinition>"
    )
    pkg.add_cache("timelineCache", "timelineCaches", xml, name, _WB_TIMELINE_CACHES)
    pkg.pivot_cache_ext(pivot, _PIVOT_CACHE_X14, _x14_cache(pivot, None, where))
    if "timeline_pivot_cache" in entry:
        attrs = _attrs(entry["timeline_pivot_cache"], set(), f"{where} timeline_pivot_cache")
        pkg.pivot_cache_ext(
            pivot,
            _PIVOT_CACHE_TIMELINE,
            f'<ext uri="{_PIVOT_CACHE_TIMELINE}" xmlns:x15="{X15_NS}"><x15:timelinePivotCacheDefinition{attrs}/></ext>',
        )
    attrs = _attrs(entry.get("timeline"), {"name", "cache", "caption"}, f"{where} timeline")
    if "level" not in attrs:
        attrs += ' level="2" selectionLevel="2"'
    shape = pkg.unique(caption, " ")
    return (
        f"<timeline name={quoteattr(shape)} cache={quoteattr(name)} caption={quoteattr(caption)}{attrs}/>",
        shape,
    )


def _frame(k: int, anchor: str, shape: str, kind: str) -> str:
    """One drawing frame: a slicer (`a14`), a table slicer (`sle15`) or a timeline (`tsle`), as Excel wraps it."""
    row, col = xlsxwriter.utility.xl_cell_to_rowcol(anchor.replace("$", ""))
    requires, namespace = {
        "slicer": ("a14", _A14_NS),
        "table": ("sle15", _SLE15_NS),
        "timeline": ("tsle", _TSLE_NS),
    }[kind]
    graphic = (
        f'<a:graphicData uri="{_TSLE_NS}"><tsle:timeslicer name={quoteattr(shape)}/></a:graphicData>'
        if kind == "timeline"
        else f'<a:graphicData uri="{_SLE_NS}"><sle:slicer xmlns:sle="{_SLE_NS}" name={quoteattr(shape)}/></a:graphicData>'
    )
    return (
        f'<xdr:twoCellAnchor editAs="oneCell"><xdr:from><xdr:col>{col}</xdr:col><xdr:colOff>0</xdr:colOff>'
        f"<xdr:row>{row}</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:from><xdr:to><xdr:col>{col + 3}</xdr:col>"
        f"<xdr:colOff>0</xdr:colOff><xdr:row>{row + 10}</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:to>"
        f'<mc:AlternateContent xmlns:mc="{_MC_NS}"><mc:Choice xmlns:{requires}="{namespace}" Requires="{requires}">'
        f'<xdr:graphicFrame macro=""><xdr:nvGraphicFramePr><xdr:cNvPr id="{k + 1}" name={quoteattr(shape)}/>'
        '<xdr:cNvGraphicFramePr/></xdr:nvGraphicFramePr><xdr:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/>'
        f"</xdr:xfrm><a:graphic>{graphic}</a:graphic></xdr:graphicFrame></mc:Choice></mc:AlternateContent>"
        "<xdr:clientData/></xdr:twoCellAnchor>"
    )


def _sheet_parts(number: int, sheet: dict[str, Any], pkg: _Package) -> tuple[list[str], str, str] | None:
    """A sheet's slicers and timelines parts and drawing; returns its relationships, extensions and drawing
    relationship id, or None when it has neither."""
    slicers: dict[bool, list[str]] = {False: [], True: []}
    timelines: list[str] = []
    frames: list[str] = []
    for entry in _list(sheet, "slicers"):
        element, shape, table = _slicer(entry, pkg, sheet["name"])
        slicers[table].append(element)
        frames.append(_frame(len(frames) + 1, str(entry["anchor"]), shape, "table" if table else "slicer"))
    for entry in _list(sheet, "timelines"):
        element, shape = _timeline(entry, pkg, sheet["name"])
        timelines.append(element)
        frames.append(_frame(len(frames) + 1, str(entry["anchor"]), shape, "timeline"))
    if not frames:
        return None
    rels: list[str] = []
    exts: list[str] = []
    for kind, elements, uri, suffix in (
        ("slicer", slicers[False], _SHEET_SLICERS, ""),
        ("slicer", slicers[True], _SHEET_TABLE_SLICERS, "t"),
        ("timeline", timelines, _SHEET_TIMELINES, ""),
    ):
        if not elements:
            continue
        rid = f"rIdSlicer{number}_{len(rels) + 1}"
        part = f"{kind}s/{kind}{number}{suffix}.xml"
        ns = X15_NS if kind == "timeline" else X14_NS
        pkg.added[f"xl/{part}"] = xml_part(f'<{kind}s xmlns="{ns}" {_ROOT}>{"".join(elements)}</{kind}s>')
        pkg.types.append(
            f'<Override PartName="/xl/{part}" ContentType="application/vnd.ms-excel.{kind}+xml"/>'
        )
        year = "2007" if kind == "slicer" else "2011"
        rels.append(
            f'<Relationship Id="{rid}" Type="{_MS_REL}/{year}/relationships/{kind}" Target="../{part}"/>'
        )
        if kind == "timeline":
            exts.append(
                f'<ext uri="{uri}" xmlns:x15="{X15_NS}"><x15:timelineRefs><x15:timelineRef r:id="{rid}"/>'
                "</x15:timelineRefs></ext>"
            )
        elif suffix:
            exts.append(
                f'<ext uri="{uri}" xmlns:x15="{X15_NS}"><x14:slicerList xmlns:x14="{X14_NS}">'
                f'<x14:slicer r:id="{rid}"/></x14:slicerList></ext>'
            )
        else:
            exts.append(
                f'<ext uri="{uri}" xmlns:x14="{X14_NS}"><x14:slicerList><x14:slicer r:id="{rid}"/></x14:slicerList></ext>'
            )
    drawing = f"drawings/drawing{number}.xml"
    pkg.added[f"xl/{drawing}"] = xml_part(
        f'<xdr:wsDr xmlns:xdr="{_XDR_NS}" xmlns:a="{_A_NS}">{"".join(frames)}</xdr:wsDr>'
    )
    pkg.types.append(f'<Override PartName="/xl/{drawing}" ContentType="{_DRAWING_TYPE}"/>')
    rid = f"rIdSlicerDrawing{number}"
    rels.append(f'<Relationship Id="{rid}" Type="{REL_NS}/drawing" Target="../{drawing}"/>')
    return rels, "".join(exts), rid


def _sheet_patch(drawing_rid: str, exts: str) -> Callable[[str], str]:
    """The sheet's `<drawing>` and its extLst entries (appended to the sheet's extLst, or a new one)."""
    insert = _insert_drawing(drawing_rid)

    def apply(text: str) -> str:
        text = insert(text)
        if "</extLst></worksheet>" in text:
            return text.replace("</extLst></worksheet>", exts + "</extLst></worksheet>")
        return text.replace("</worksheet>", f"<extLst>{exts}</extLst></worksheet>")

    return apply


def _list(sheet: dict[str, Any], key: str) -> list[Any]:
    entries = sheet.get(key, [])
    if not isinstance(entries, list):
        raise ValueError(f"sheet {sheet['name']}: {key} is a list")
    return entries


def _workbook_patch(pkg: _Package) -> Callable[[str], str]:
    """The workbook's defined names and extLst (slicer caches, timeline caches, table slicer caches, in the
    order Excel writes the three extensions)."""
    lists = {_WB_SLICER_CACHES: "", _WB_TIMELINE_CACHES: "", _WB_TABLE_SLICER_CACHES: ""}
    for uri, rid in pkg.cache_refs:
        tag = "x15:timelineCacheRef" if uri == _WB_TIMELINE_CACHES else "x14:slicerCache"
        lists[uri] += f'<{tag} r:id="{rid}"/>'
    exts = ""
    for uri, (ns, wrapper, extra) in {
        _WB_SLICER_CACHES: ("x14", "x14:slicerCaches", ""),
        _WB_TIMELINE_CACHES: ("x15", "x15:timelineCacheRefs", ""),
        _WB_TABLE_SLICER_CACHES: ("x15", "x15:slicerCaches", f' xmlns:x14="{X14_NS}"'),
    }.items():
        if lists[uri]:
            space = X14_NS if ns == "x14" else X15_NS
            exts += f'<ext uri="{uri}" xmlns:{ns}="{space}"><{wrapper}{extra}>{lists[uri]}</{wrapper}></ext>'
    names = "".join(pkg.names)

    def apply(text: str) -> str:
        if "<extLst>" in text:
            raise ValueError("cannot place slicer caches: the workbook already has an extLst")
        if "</definedNames>" in text:
            text = text.replace("</definedNames>", names + "</definedNames>")
        else:
            text = text.replace("<calcPr", f"<definedNames>{names}</definedNames><calcPr", 1)
        return text.replace("</workbook>", f"<extLst>{exts}</extLst></workbook>")

    return apply


def _cache_patch(exts: dict[str, str]) -> Callable[[str], str]:
    """The pivot cache's extLst: x14 (pivotCacheId) first, then the x15 timeline extension."""
    ordered = "".join(exts[uri] for uri in (_PIVOT_CACHE_X14, _PIVOT_CACHE_TIMELINE) if uri in exts)

    def apply(text: str) -> str:
        if "<extLst>" in text:
            raise ValueError("cannot place a slicer's pivot cache extension: the cache already has an extLst")
        return text.replace("</pivotCacheDefinition>", f"<extLst>{ordered}</extLst></pivotCacheDefinition>")

    return apply


def _tables(parts: dict[str, bytes]) -> dict[str, tuple[int, list[str]]]:
    """Each table of the package by name: its id and its column names."""
    out = {}
    for part, data in parts.items():
        if re.fullmatch(r"xl/tables/table\d+\.xml", part):
            text = data.decode()
            head = re.search(r'<table [^>]*?id="(\d+)" name="([^"]+)"', text)
            if head:
                out[head[2]] = (int(head[1]), re.findall(r'<tableColumn id="\d+" name="([^"]+)"', text))
    return out


def write_slicers(xlsx: bytes, spec: dict[str, Any]) -> bytes:
    """Adds each sheet's `slicers` and `timelines` to the package `xlsx` (spec2xlsx's last pivot step)."""
    if not any(sheet.get("slicers") or sheet.get("timelines") for sheet in spec["sheets"]):
        return xlsx
    parts = read_parts(xlsx)
    pkg = _Package(_pivots(parts, spec), _tables(parts))
    patches: list[tuple[str, Callable[[str], str]]] = []
    for number, sheet in enumerate(spec["sheets"], start=1):
        made = _sheet_parts(number, sheet, pkg)
        if made is None:
            continue
        rels, exts, rid = made
        rels_part = f"xl/worksheets/_rels/sheet{number}.xml.rels"
        if rels_part in parts:
            patches.append((rels_part, _append("</Relationships>", "".join(rels))))
        else:
            pkg.added[rels_part] = relationships_xml(*rels)
        patches.append((f"xl/worksheets/sheet{number}.xml", _sheet_patch(rid, exts)))
    patches.extend((part, _cache_patch(exts)) for part, exts in pkg.cache_ext.items())
    patches.append(("xl/workbook.xml", _workbook_patch(pkg)))
    patches.append(("xl/_rels/workbook.xml.rels", _append("</Relationships>", "".join(pkg.workbook_rels))))
    patches.append(("[Content_Types].xml", _append("</Types>", "".join(pkg.types))))
    return patch_parts(xlsx, patches, pkg.added)
