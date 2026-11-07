"""Pivot tables defined field by field in spec cases (T-4122, case builder v2; decision 0274).

The original builder (pivot.py) writes one compact shape and renders its cells itself. A pivot with a
`"definition"` is different: the builder writes only the package parts and **Excel renders the pivot**
when it opens the file (the cache is `refreshOnLoad`; docs/research/pivot-tables-2026-11-04/
oracle-spike.md runs 1, 4-6). The definition maps one-to-one onto `pivotTableDefinition`:

    {"name": "Pv", "source": "Data!A1:D9", "location": "A3",
     "definition": {
       "table": {"outline": false, "compact": false},           # pivotTableDefinition attributes
       "rows": ["Region", "Product"],                           # rowFields, outer first
       "columns": ["Values"],                                   # colFields; "Values" is the Σ field (x=-2)
       "pages": [{"field": "Year", "item": 2024,                # pageFields; "item" omitted is (All)
                  "attrs": {"cap": "Y"}}],                     # other `<pageField>` attributes (T-4130)
       "data": [{"field": "Sales"},                             # dataFields
                {"field": "Sales", "subtotal": "count", "name": "N"},
                {"field": "Sales", "showDataAs": "percentOfTotal", "numFmtId": 10},
                {"field": "Sales", "showDataAs": "rankDescending", "baseField": "Region"}],  # x14 pivotShowAs
       "fields": {"Product": {"attrs": {"sortType": "descending"},   # pivotField attributes
                              "hidden": ["C"], "collapsed": ["A"],   # item h="1" / sd="0"
                              "subtotals": ["sum", "count"],         # custom subtotals (none: [])
                              "group": {"start": 0, "end": 100, "interval": 10},   # numeric fieldGroup
                              "discrete": {"name": "Product2", "groups": [...]},  # manual groups (T-4170)
                              "item_attrs": {"B": {"d": True}},          # other `<item>` attributes
                              "item_order": ["C", "A"],              # `<item>` order only (T-4164)
                              "xml": {"autoSortScope": "<autoSortScope>...</autoSortScope>",  # raw children
                                      "extLst": "<extLst>...</extLst>",   # after `items` (T-4130)
                                      "groupItems": "<groupItems>...</groupItems>"}}},  # its group's (T-4767)
       "calculated": [{"name": "Double", "formula": "Sales*2"}],    # calculated cache fields
       "calculated_items": [{"field": "Product", "name": "AB",       # calculated items (T-4130)
                             "formula": "A+B", "attrs": {}, "pivot_area": "<pivotArea>...</pivotArea>"}],
       "cache": {"missingItemsLimit": 0},                       # pivotCacheDefinition attributes
       "cache_names": ["Region", "Sales", "Sales_b", "Blank"],  # cacheField names in place of the header
       "records_xml": [{"record": 0, "field": "Sales",          # a record value, raw (T-4767: tuples)
                        "xml": "<n v=\\"1\\"><tpls c=\\"1\\">...</tpls></n>"}],
       "xml": {"filters": "<filters count=\\"1\\">...</filters>"},    # raw children, by slot
       "refresh_on_load": false,                                # the cache's refreshOnLoad (default true)
       "area": "A3:B6"}}                                        # the <location> ref (default A3:B4)

Field indices (`fld` in raw XML, `baseField`) count the source columns left to right, then the calculated
fields in order, or the fields that discrete and date-level groups add (pivot_groups.py, T-4170: `discrete`
and `date_group` field entries). Excel for the web refuses a package whose parts disagree with each other (run 2:
`OpenWorkbookBlockedWorkbook`), so the builder derives everything that must agree (shared items, records,
`<items>`, counts) from the source cells and refuses what Excel would refuse or crash on (ClosedXML's
invariants, docs/research/pivot-tables-2026-11-04/oss-engines.md §4). It does not need to agree with what
Excel will compute: Excel rebuilds the cache, the items and the layout on open. Source formulas are cached as
blank for that reason.

The cache's field names are the source header, which must be distinct non-empty texts. `cache_names` saves
other names instead, one per source column, so a header Excel names itself on refresh (a blank cell, a
repeated constant; T-4627) can be given; the definition refers to the fields by these names.

The pivot's area must hold no spec cell: a layout that meets a foreign cell collapses to `#SPILL!` (run 3).
The builder checks the anchor and the page-field area; the case author keeps the rest of the area empty.

A pivot with `"refresh_on_load": false` (T-4586) is saved without `refreshOnLoad`, as most files Excel
saves are. Its area may then hold spec cells: they are the cells the file saved for the pivot. With `area`
(its `<location>` ref) and the raw `rowItems` and `colItems` slots of `xml`, a case can save a render that
disagrees with the cache, to ask whether Excel keeps it on open or renders the pivot again.

The eval adapter writes the same parts in Rust (crates/rustsheet-eval/src/pivot_definition.rs, T-4152), so the
engine loads what Excel opened: keep the two in step. tests/test_pivot_parts_golden.py and the Rust module's
tests pin both to the golden parts in tests/pivot_parts_golden/.
"""

from __future__ import annotations

import math
import re
from dataclasses import dataclass, field
from typing import Any
from xml.sax.saxutils import quoteattr

import xlsxwriter.utility

from rustsheet_tools import pivot_groups
from rustsheet_tools.excel_errors import ERRORS
from rustsheet_tools.pivot_xml import MAIN_NS, REL_NS, date_shared_items, iso_date, number_text

VALUES = "Values"  # the Σ (data) field on an axis: `<field x="-2"/>`
_KEYS = {"name", "source", "location", "definition"}
_DEFINITION_KEYS = {
    "table",
    "rows",
    "columns",
    "pages",
    "data",
    "fields",
    "calculated",
    "cache",
    "cache_names",
    "xml",
    "refresh_on_load",
    "area",
    "calculated_items",
    "records_xml",
}
_FIELD_KEYS = {
    "attrs",
    "hidden",
    "collapsed",
    "subtotals",
    "order",
    "item_order",
    "item_attrs",
    "xml",
    *pivot_groups.GROUPINGS,
}
# `<item>` attributes the builder writes itself (`x`, and `h`/`sd` from `hidden`/`collapsed`), which
# `item_attrs` may not set; `t` marks a subtotal item, not a value item.
_MANAGED_ITEM = {"x", "h", "sd", "t"}
_DATA_KEYS = {"field", "name", "subtotal", "showDataAs", "baseField", "baseItem", "numFmtId"}
# ST_DataConsolidateFunction, with the verb Excel puts in a data field's default name.
_AGGREGATE_VERBS = [
    "sum",
    "Sum",
    "count",
    "Count",
    "average",
    "Average",
    "max",
    "Max",
    "min",
    "Min",
    "product",
    "Product",
    "countNums",
    "Count",
    "stdDev",
    "StdDev",
    "stdDevp",
    "StdDevp",
    "var",
    "Var",
    "varp",
    "Varp",
]
AGGREGATES = dict(zip(_AGGREGATE_VERBS[::2], _AGGREGATE_VERBS[1::2], strict=True))
# A pivotField's custom subtotal functions: the ST_ItemType names its `<item t>` and `<name>Subtotal`
# attributes use, which differ from ST_DataConsolidateFunction (`countA` is not `count`; T-4622).
# Keep in step with `SUBTOTAL_FUNCTIONS` in crates/rustsheet-xlsx/src/import/pivots/table.rs.
SUBTOTALS = ("sum", "countA", "avg", "max", "min", "product", "count", "stdDev", "stdDevP", "var", "varP")
# ST_ShowDataAs (ECMA-376) plus the Excel 2010 modes (`SHOW_AS_X14`), which Excel writes as x14
# `pivotShowAs` in the data field's extLst with no `showDataAs` (excel-docs.md §1.4); accepted so probes
# can ask whether Excel for the web honours them (T-4129).
SHOW_AS_X14 = (
    "percentOfParent",
    "percentOfParentRow",
    "percentOfParentCol",
    "percentOfRunningTotal",
    "rankAscending",
    "rankDescending",
)
X14_NS = "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"
# The x14 dataField extension's uri ([MS-XLSX] 2.2.4.4).
X14_DATA_FIELD_URI = "{E15A36E0-9728-4e99-A89B-3F7291B0FE68}"
SHOW_DATA_AS = {
    *SHOW_AS_X14,
    "normal",
    "difference",
    "percent",
    "percentDiff",
    "runTotal",
    "percentOfRow",
    "percentOfCol",
    "percentOfTotal",
    "index",
}
# pivotTableDefinition attributes written unless `table` sets them (Excel's own new-pivot values).
_TABLE_DEFAULTS = {
    "dataCaption": "Values", "updatedVersion": "6", "minRefreshableVersion": "3", "createdVersion": "6",
    "applyNumberFormats": "0", "applyBorderFormats": "0", "applyFontFormats": "0", "applyPatternFormats": "0",
    "applyAlignmentFormats": "0", "applyWidthHeightFormats": "1", "useAutoFormatting": "1",
    "itemPrintTitles": "1", "indent": "0", "outline": "1", "outlineData": "1",
}  # fmt: skip
# Builder-managed attributes an author may not set through `table`, `attrs` or `cache`.
_MANAGED_TABLE = {"name", "cacheId", "dataOnRows"}
_MANAGED_FIELD = {
    "axis",
    "dataField",
    "showAll",
    "defaultSubtotal",
    "multipleItemSelectionAllowed",
    *(f"{a}Subtotal" for a in SUBTOTALS),
}
_MANAGED_CACHE = {"refreshOnLoad", "recordCount", "id"}
# `<pageField>` attributes the builder writes itself.
_MANAGED_PAGE = {"fld", "item", "hier"}
# `<calculatedItem>` attributes the builder writes itself.
_MANAGED_CALCULATED_ITEM = {"formula"}
_CALCULATED_ITEM_KEYS = {"field", "name", "formula", "attrs", "pivot_area"}
# A pivotField's children after `items`, in schema order: its raw-XML slots (T-4129, T-4130).
_FIELD_SLOTS = ("autoSortScope", "extLst")
# The value elements a `records_xml` entry may write (CT_Record's children).
_RECORD_TAGS = ("m", "n", "b", "e", "s", "d", "x")
# A grouped field's raw cache slot (T-4767): its `<groupItems>`, in place of the builder's in its `<fieldGroup>`.
_GROUP_SLOT = "groupItems"
# CT_pivotTableDefinition's children after `dataFields`, in schema order: the raw-XML slots.
_SLOTS = ("formats", "conditionalFormats", "chartFormats", "pivotHierarchies", "filters", "extLst")
# Saved layout lines (T-4586), raw: each goes right after its axis's fields element, in schema order.
_AXIS_SLOTS = ("rowItems", "colItems")
_ATTR_NAME = re.compile(r"[A-Za-z][A-Za-z0-9]*")
_AREA = re.compile(r"[A-Z]{1,3}[1-9][0-9]{0,6}:[A-Z]{1,3}[1-9][0-9]{0,6}")

# A cached value: text, number, boolean, ("e", error text) or None (blank or not known to the builder).
Value = str | float | bool | tuple[str, str] | None


def _attr_value(value: Any) -> str:
    if isinstance(value, bool):
        return "1" if value else "0"
    if isinstance(value, int | float) and not isinstance(value, bool):
        return number_text(float(value))
    if isinstance(value, str):
        return value
    raise ValueError(f"attribute value {value!r} must be a string, number or boolean")


def _attrs(raw: Any, managed: set[str], where: str) -> str:
    """` k="v"` for each author attribute, refusing bad names and builder-managed ones."""
    if raw is None:
        return ""
    if not isinstance(raw, dict):
        raise ValueError(f"{where} must be an object of attributes")
    out = []
    for key, value in raw.items():
        if not _ATTR_NAME.fullmatch(key) or key in managed:
            raise ValueError(f"{where} may not set attribute {key!r}")
        out.append(f" {key}={quoteattr(_attr_value(value))}")
    return "".join(out)


def cached_value(raw: Any) -> Value:
    """The value a spec cell contributes to a pivot cache."""
    if raw is None or raw == "" or (isinstance(raw, str) and raw.startswith("=")):
        return None  # blank, `=""`, or a formula: Excel recomputes the cache on open
    if isinstance(raw, bool | str):
        return raw
    if isinstance(raw, int | float):
        if not math.isfinite(raw):
            raise ValueError(f"pivot source value {raw!r} is not finite")
        return float(raw)
    if isinstance(raw, dict) and raw.get("t") == "e" and raw.get("v") in ERRORS:
        return ("e", str(raw["v"]))
    if isinstance(raw, dict) and raw.get("t") == "s" and isinstance(raw.get("v"), str):
        return str(raw["v"])
    if isinstance(raw, dict) and ("entered" in raw or "entered_r1c1" in raw):
        return None  # typed input: Excel parses it after load
    raise ValueError(f"pivot source cell {raw!r} is not a value the builder can cache")


def fold(text: str) -> str:
    """How the builder compares texts without regard to case (item identity, field names): `str.lower`, as the
    Rust mirror's `to_lowercase`, so both builders agree past ASCII: `SS` is `ss`, but `ß` is not `ss` (casefold
    would make it so, and the mirror would not)."""
    return text.lower()


def _key(value: Value) -> tuple[str, str]:
    """Identity of a cache item for the builder's shared-items list. Texts that differ only in case are
    one item: Excel for the web refuses a cache listing both as corrupt (T-4122 runs). Excel rebuilds the
    items on open, so which casing it keeps is its own rule (T-4134), not this one."""
    if isinstance(value, bool):
        return ("b", str(value))
    if isinstance(value, float):
        return ("n", repr(value))
    if isinstance(value, tuple):
        return ("e", value[1])
    return ("m", "") if value is None else ("s", fold(value))


def _item_xml(value: Value) -> str:
    if value is None:
        return "<m/>"
    if isinstance(value, bool):
        return f'<b v="{int(value)}"/>'
    if isinstance(value, float):
        return f'<n v="{number_text(value)}"/>'
    if isinstance(value, tuple):
        return f"<e v={quoteattr(value[1])}/>"
    return f"<s v={quoteattr(value)}/>"


@dataclass
class _Field:
    """One cache field and the pivot's use of it."""

    name: str
    values: list[Value]  # per record; empty for a calculated field
    formula: str | None = None
    items: list[Value] = field(default_factory=list)  # distinct values, first-seen order
    spec: dict[str, Any] = field(default_factory=dict)  # its `fields` entry
    axis: str | None = None  # axisRow, axisCol, axisPage
    data: bool = False
    calculated: set[tuple[str, str]] = field(default_factory=set)  # keys of its calculated items
    # Discrete and date-level groups (pivot_groups.py, T-4170).
    added: bool = False  # a field a group adds (`databaseField="0"`)
    labels: list[Value] | None = None  # the group items its pivotField indexes
    item_order: list[int] | None = None  # its pivotField's item order (indices into `labels`)
    field_group: str = ""  # its `<fieldGroup>`, after its shared items
    dates: bool = False  # grouped by date: cached as dates
    discrete_base: bool = False  # the base of a discrete group: its shared items are listed

    def __post_init__(self) -> None:
        seen: set[tuple[str, str]] = set()
        for v in self.values:
            if _key(v) not in seen:
                seen.add(_key(v))
                self.items.append(v)

    def reorder(self, order: Any, pivot: str) -> None:
        """Puts the items `order` lists first, in that order: Excel keeps a field's item order on refresh
        (T-4122 runs), so the order is part of what a case asks Excel."""
        if not isinstance(order, list):
            raise ValueError(f"pivot {pivot}: order of {self.name!r} is a list of items")
        keys = [_key(cached_value(raw)) for raw in order]
        by_key = {_key(v): v for v in self.items}
        if missing := [raw for raw, k in zip(order, keys, strict=True) if k not in by_key]:
            raise ValueError(f"pivot {pivot}: order items {missing!r} are not in field {self.name!r}")
        if len(set(keys)) != len(keys):
            raise ValueError(f"pivot {pivot}: order of {self.name!r} repeats an item")
        self.items = [by_key[k] for k in keys] + [v for v in self.items if _key(v) not in set(keys)]

    @property
    def numeric_only(self) -> bool:
        return all(isinstance(v, float) or v is None for v in self.values)

    @property
    def group(self) -> dict[str, Any] | None:
        group = self.spec.get("group")
        return group if isinstance(group, dict) else None

    def shared_items(self) -> str:
        """A date-grouped field writes Excel's date flags instead (`date_shared_items`, as pivot.py's)."""
        if self.dates:
            numbers = [v for v in self.values if isinstance(v, float)]
            if not numbers:
                raise ValueError(f"pivot field {self.name!r}: a date group needs a date")
            return date_shared_items(numbers, blank=len(numbers) < len(self.values))
        return self._shared_items()

    def _shared_items(self) -> str:
        """`<sharedItems>`: the flags always, the list when records refer to it by index. The flags follow
        Excel-saved caches (LibreOffice and ClosedXML fixtures, oss-engines.md §5): booleans and errors count
        as strings; `containsSemiMixedTypes` is 0 only with no string and no blank; `containsMixedTypes` marks
        strings beside numbers. Excel for the web refuses other combinations as corrupt (T-4122 runs)."""
        vs = self.values
        strings = any(isinstance(v, str | bool | tuple) for v in vs)
        numbers = [v for v in vs if isinstance(v, float)]
        blank = any(v is None for v in vs)
        flags = []
        if not strings and not blank:
            flags.append(' containsSemiMixedTypes="0"')
        if not strings:
            flags.append(' containsString="0"')
        if strings and numbers:
            flags.append(' containsMixedTypes="1"')
        if numbers:
            flags.append(' containsNumber="1"')
            if all(n.is_integer() for n in numbers):
                flags.append(' containsInteger="1"')
            flags.append(f' minValue="{number_text(min(numbers))}" maxValue="{number_text(max(numbers))}"')
        if blank:
            flags.append(' containsBlank="1"')
        if not self.listed:
            return f"<sharedItems{''.join(flags)}/>"
        items = "".join(_item_xml(v) for v in self.items)
        return f'<sharedItems{"".join(flags)} count="{len(self.items)}">{items}</sharedItems>'

    @property
    def listed(self) -> bool:
        """Whether records refer to the shared items by index (`<x v>`): every field except an all-number
        field that is not on an axis or is grouped numerically or by date (Excel's own shape). A discrete
        group's base is always listed: its `<discretePr>` indexes the shared items."""
        if self.discrete_base:
            return True
        return not (self.numeric_only and (self.axis is None or self.group is not None or self.dates))

    def group_labels(self) -> list[str]:
        """Numeric group items as the builder writes them; Excel relabels them on open (run 6)."""
        g = self.group or {}
        start, end, step = (float(g.get(k, 0)) for k in ("start", "end", "interval"))
        if not step > 0 or end < start or (end - start) / step > 10_000:
            raise ValueError(f"pivot field {self.name!r} group needs 0 < interval and at most 10,000 bins")
        bins = []
        lo = start
        while lo <= end:
            bins.append(f"{number_text(lo)}-{number_text(lo + step)}")
            lo += step
        return [f"<{number_text(start)}", *bins, f">{number_text(end)}"]


class DefinedPivot:
    """A pivot built from a `definition` (module docstring)."""

    def __init__(self, pivot: dict[str, Any], header: list[Any], rows: list[list[Any]]) -> None:
        if unknown := set(pivot) - _KEYS:
            raise ValueError(f"pivot table with a definition has unknown keys {sorted(unknown)}")
        self.name = str(pivot["name"])
        definition = pivot["definition"]
        if not isinstance(definition, dict) or (unknown := set(definition) - _DEFINITION_KEYS):
            raise ValueError(f"pivot {self.name}: definition has unknown keys {sorted(unknown or [])}")
        self.d = definition
        cache_names = definition.get("cache_names")
        if cache_names is not None:
            if not isinstance(cache_names, list) or len(cache_names) != len(header):
                raise ValueError(f"pivot {self.name}: cache_names needs one name per source column")
            header = cache_names
        if not all(isinstance(h, str) and h for h in header) or len({fold(h) for h in header}) != len(header):
            raise ValueError(f"pivot {self.name}: source header {header!r} must be distinct non-empty texts")
        self.fields = [_Field(h, [cached_value(r[i]) for r in rows]) for i, h in enumerate(header)]
        self.records = len(rows)
        for calc in definition.get("calculated", []):
            if not isinstance(calc, dict) or set(calc) != {"name", "formula"}:
                raise ValueError(f"pivot {self.name}: a calculated field is {{name, formula}}")
            self.fields.append(_Field(str(calc["name"]), [], formula=str(calc["formula"])))
        names = [fold(f.name) for f in self.fields]
        if len(set(names)) != len(names):
            raise ValueError(f"pivot {self.name}: calculated field names must differ from the source fields")
        self._read_fields(definition.get("fields") or {})
        self.calculated_items = self._calculated_items(definition.get("calculated_items", []))
        self.rows = self._axis(definition.get("rows", []), "axisRow")
        self.columns = self._axis(definition.get("columns", []), "axisCol")
        self.pages = self._pages(definition.get("pages", []))
        self.data = self._data(definition.get("data", []))
        self._values_field()
        refresh = definition.get("refresh_on_load", True)
        if not isinstance(refresh, bool):
            raise ValueError(f"pivot {self.name}: refresh_on_load is true or false")
        self.refresh_on_load = refresh
        for f in self.fields:
            if f.formula is not None and (f.axis is not None or f.spec):
                raise ValueError(f"pivot {self.name}: calculated field {f.name!r} can only be a data field")
            if f.group is not None and not (f.numeric_only and f.axis in ("axisRow", "axisCol")):
                raise ValueError(
                    f"pivot {self.name}: grouped field {f.name!r} must be numbers on the row or column axis"
                )

    def _read_fields(self, entries: Any) -> None:
        """The `fields` entries: a source field's first (with its `order`), then the groups, then the
        entries of the fields they add."""
        if not isinstance(entries, dict):
            raise ValueError(f"pivot {self.name}: fields maps field names to entries")
        added: list[tuple[str, dict[str, Any]]] = []  # entries of fields a group adds
        for name, entry in entries.items():
            if not isinstance(entry, dict) or (unknown := set(entry) - _FIELD_KEYS):
                raise ValueError(
                    f"pivot {self.name}: field {name!r} entry has unknown keys {sorted(unknown or [])}"
                )
            if (i := self._find(name)) is None:
                added.append((name, entry))
                continue
            target = self.fields[i]
            target.spec = entry
            target.reorder(entry.get("order", []), self.name)
        self._add_groups()
        for name, entry in added:
            target = self.fields[self._index(name)]
            if not target.added or set(entry) - pivot_groups.ADDED_FIELD_KEYS:
                raise ValueError(
                    f"pivot {self.name}: an added group field's entry takes only "
                    f"{sorted(pivot_groups.ADDED_FIELD_KEYS)}: {name!r}"
                )
            target.spec = entry

    def _add_groups(self) -> None:
        """Discrete and date-level groups (pivot_groups.py): each groups its field and appends the fields
        it adds, in source-field order."""
        for i, f in enumerate(list(self.fields)):
            kinds = [k for k in pivot_groups.GROUPINGS if k in f.spec]
            if len(kinds) > 1:
                raise ValueError(
                    f"pivot {self.name}: field {f.name!r} takes one of {list(pivot_groups.GROUPINGS)}"
                )
            if kinds in ([], ["group"]):
                continue
            if self.d.get("calculated") or self.d.get("calculated_items"):
                raise ValueError(
                    f"pivot {self.name}: calculated fields and items do not combine with discrete or date groups"
                )
            where = f"pivot {self.name} field {f.name!r}"
            if kinds == ["discrete"]:
                group = pivot_groups.discrete(f.spec["discrete"], f.items, _key, cached_value, where)
                f.discrete_base = True
                f.field_group = f'<fieldGroup par="{len(self.fields)}"/>'
                self.fields.append(
                    _Field(
                        group.name,
                        [],
                        items=list(group.labels),
                        added=True,
                        labels=list(group.labels),
                        item_order=group.order,
                        field_group=pivot_groups.discrete_xml(i, group, _item_xml),
                    )
                )
                continue
            dates = pivot_groups.date_group(f.spec["date_group"], where)
            if not f.numeric_only:
                raise ValueError(f"{where}: a date group's field holds date serials")
            f.dates = True
            first, *more = dates.levels
            f.labels = list(pivot_groups.date_labels(first, dates.start, dates.end))
            for by in more:
                labels: list[Value] = list(pivot_groups.date_labels(by, dates.start, dates.end))
                self.fields.append(
                    _Field(
                        pivot_groups.LEVEL_NAMES[by],
                        [],
                        items=list(labels),
                        added=True,
                        labels=labels,
                        field_group=pivot_groups.date_xml(by, dates, i, None),
                    )
                )
            f.field_group = pivot_groups.date_xml(first, dates, i, len(self.fields) - 1 if more else None)
        names = [fold(f.name) for f in self.fields]
        if len(set(names)) != len(names):
            raise ValueError(f"pivot {self.name}: group field names must differ from the other fields")

    def _calculated_items(self, raw: Any) -> list[dict[str, Any]]:
        """Calculated items (T-4130): each is appended to its field's shared items, so its `<item>` and its
        `pivotArea` reference can index it, and its pivotField item is written `f="1"`."""
        if not isinstance(raw, list):
            raise ValueError(f"pivot {self.name}: calculated_items is a list")
        out = []
        for entry in raw:
            if (
                not isinstance(entry, dict)
                or set(entry) - _CALCULATED_ITEM_KEYS
                or not {"field", "name", "formula"} <= set(entry)
            ):
                raise ValueError(
                    f"pivot {self.name}: a calculated item is {{field, name, formula, attrs?, pivot_area?}}"
                )
            i = self._index(entry["field"])
            f = self.fields[i]
            name = str(entry["name"])
            if f.formula is not None or not name or _key(name) in {_key(v) for v in f.items}:
                raise ValueError(
                    f"pivot {self.name}: calculated item {name!r} needs a source field without that item"
                )
            f.items.append(name)
            f.calculated.add(_key(name))
            out.append({**entry, "fld": i, "x": len(f.items) - 1})
        return out

    def _find(self, name: Any) -> int | None:
        for i, f in enumerate(self.fields):
            if isinstance(name, str) and fold(f.name) == fold(name):
                return i
        return None

    def _index(self, name: Any) -> int:
        i = self._find(name)
        if i is None:
            raise ValueError(f"pivot {self.name}: no field {name!r}")
        return i

    def _axis(self, names: Any, axis: str) -> list[int]:
        if not isinstance(names, list):
            raise ValueError(f"pivot {self.name}: rows and columns are lists of field names")
        out = []
        for name in names:
            if name == VALUES:
                out.append(-2)
                continue
            i = self._index(name)
            if self.fields[i].axis is not None:
                raise ValueError(f"pivot {self.name}: field {name!r} is on two axes")
            self.fields[i].axis = axis
            out.append(i)
        if out.count(-2) > 1:
            raise ValueError(f"pivot {self.name}: the Values field appears twice")
        return out

    def _pages(self, pages: Any) -> list[tuple[int, Value | None, str]]:
        if not isinstance(pages, list):
            raise ValueError(f"pivot {self.name}: pages is a list")
        out: list[tuple[int, Value | None, str]] = []
        for page in pages:
            if (
                not isinstance(page, dict)
                or not set(page) <= {"field", "item", "attrs"}
                or "field" not in page
            ):
                raise ValueError(f"pivot {self.name}: a page is {{field, item?, attrs?}}")
            i = self._index(page["field"])
            if self.fields[i].axis is not None or self.fields[i].formula is not None or self.fields[i].dates:
                raise ValueError(
                    f"pivot {self.name}: page field {page['field']!r} is already used or date-grouped"
                )
            self.fields[i].axis = "axisPage"
            item = cached_value(page["item"]) if "item" in page else None
            if "item" in page and _key(item) not in {_key(v) for v in self.fields[i].items}:
                raise ValueError(
                    f"pivot {self.name}: page item {page['item']!r} is not in field {page['field']!r}"
                )
            extra = _attrs(
                page.get("attrs"), _MANAGED_PAGE, f"pivot {self.name} page {page['field']!r} attrs"
            )
            out.append((i, item if "item" in page else None, extra))
        return out

    def _data(self, data: Any) -> list[dict[str, Any]]:
        if not isinstance(data, list) or not data:
            raise ValueError(f"pivot {self.name}: data lists at least one data field")
        out = []
        for entry in data:
            if not isinstance(entry, dict) or (unknown := set(entry) - _DATA_KEYS) or "field" not in entry:
                raise ValueError(
                    f"pivot {self.name}: a data field is {{field, ...}} without {sorted(unknown or [])}"
                )
            i = self._index(entry["field"])
            if self.fields[i].added:
                raise ValueError(f"pivot {self.name}: group field {entry['field']!r} is not a data field")
            subtotal = str(entry.get("subtotal", "sum"))
            if subtotal not in AGGREGATES:
                raise ValueError(
                    f"pivot {self.name}: subtotal {subtotal!r} is not one of {sorted(AGGREGATES)}"
                )
            show = entry.get("showDataAs")
            if show is not None and show not in SHOW_DATA_AS:
                raise ValueError(
                    f"pivot {self.name}: showDataAs {show!r} is not one of {sorted(SHOW_DATA_AS)}"
                )
            self.fields[i].data = True
            name = str(entry.get("name") or f"{AGGREGATES[subtotal]} of {self.fields[i].name}")
            out.append({**entry, "fld": i, "subtotal": subtotal, "name": name})
        names = [fold(e["name"]) for e in out]
        if len(set(names)) != len(names):
            raise ValueError(f"pivot {self.name}: data field names must differ")
        return out

    def _values_field(self) -> None:
        """Excel crashes on a Values field with one data field and needs one with several
        (ClosedXML's invariants): add it to the columns when several data fields leave it out."""
        placed = -2 in self.rows or -2 in self.columns
        if len(self.data) == 1 and placed:
            raise ValueError(f"pivot {self.name}: the Values field needs two or more data fields")
        if len(self.data) > 1 and not placed:
            self.columns.append(-2)

    # ------------------------------------------------------------------ parts
    def footprint(self, top: int, left: int) -> set[tuple[int, int]]:
        """Cells that must hold no spec cell: the anchor and the page-field area above it."""
        cells = {(top, left)}
        if self.pages:
            if top < len(self.pages) + 1:
                raise ValueError(
                    f"pivot {self.name}: page fields need {len(self.pages) + 1} free rows above the location"
                )
            for k in range(len(self.pages)):
                cells |= {(top - len(self.pages) - 1 + k, left), (top - len(self.pages) - 1 + k, left + 1)}
        return cells

    def cache_definition(self, source_sheet: str, source_ref: str) -> str:
        fields = []
        for f in self.fields:
            if f.formula is not None:
                fields.append(
                    f'<cacheField name={quoteattr(f.name)} numFmtId="0" formula={quoteattr(f.formula)}'
                    ' databaseField="0"/>'
                )
                continue
            if f.added:
                fields.append(
                    f'<cacheField name={quoteattr(f.name)} numFmtId="0" databaseField="0">{self._group_items(f, f.field_group)}</cacheField>'
                )
                continue
            group = f.field_group
            if f.group is not None:
                g = f.group
                labels = f.group_labels()
                items = "".join(f"<s v={quoteattr(label)}/>" for label in labels)
                group = (
                    f'<fieldGroup base="{self.fields.index(f)}"><rangePr autoStart="0" autoEnd="0"'
                    f' startNum="{number_text(float(g["start"]))}" endNum="{number_text(float(g["end"]))}"'
                    f' groupInterval="{number_text(float(g["interval"]))}"/>'
                    f'<groupItems count="{len(labels)}">{items}</groupItems></fieldGroup>'
                )
            fields.append(
                f'<cacheField name={quoteattr(f.name)} numFmtId="0">{f.shared_items()}{self._group_items(f, group)}</cacheField>'
            )
        extra = _attrs(self.d.get("cache"), _MANAGED_CACHE, f"pivot {self.name} cache")
        calculated = ""
        if self.calculated_items:
            entries = "".join(self._calculated_item(e) for e in self.calculated_items)
            calculated = f'<calculatedItems count="{len(self.calculated_items)}">{entries}</calculatedItems>'
        return (
            f'<pivotCacheDefinition xmlns="{MAIN_NS}" xmlns:r="{REL_NS}" r:id="rId1"'
            + (' refreshOnLoad="1"' if self.refresh_on_load else "")
            + f' createdVersion="6" refreshedVersion="6" minRefreshableVersion="3" recordCount="{self.records}"{extra}>'
            f'<cacheSource type="worksheet"><worksheetSource ref="{source_ref}" sheet={quoteattr(source_sheet)}/>'
            f'</cacheSource><cacheFields count="{len(fields)}">{"".join(fields)}</cacheFields>{calculated}'
            "</pivotCacheDefinition>"
        )

    def _calculated_item(self, e: dict[str, Any]) -> str:
        """One `<calculatedItem>`: its `pivot_area` as written, or the one Excel saves, which names the item."""
        attrs = _attrs(e.get("attrs"), _MANAGED_CALCULATED_ITEM, f"pivot {self.name} calculated item attrs")
        area = (
            self._raw(e, "pivot_area", tag="pivotArea")
            if "pivot_area" in e
            else (
                '<pivotArea cacheIndex="1" outline="0" fieldPosition="0"><references count="1">'
                f'<reference field="{e["fld"]}" count="1" selected="0"><x v="{e["x"]}"/></reference>'
                "</references></pivotArea>"
            )
        )
        return f"<calculatedItem formula={quoteattr(str(e['formula']))}{attrs}>{area}</calculatedItem>"

    def cache_records(self) -> str:
        database = [f for f in self.fields if f.formula is None and not f.added]
        index = [{_key(v): k for k, v in enumerate(f.items)} for f in database]
        raw = self._records_xml([f.name for f in database])
        rows = []
        for r in range(self.records):
            cells = []
            for k, (f, at) in enumerate(zip(database, index, strict=True)):
                v = f.values[r]
                if (r, k) in raw:
                    cells.append(raw[(r, k)])
                elif f.listed:
                    cells.append(f'<x v="{at[_key(v)]}"/>')
                elif f.dates and isinstance(v, float):
                    cells.append(f'<d v="{iso_date(v)}"/>')
                else:
                    cells.append(_item_xml(v))
            rows.append(f"<r>{''.join(cells)}</r>")
        return (
            f'<pivotCacheRecords xmlns="{MAIN_NS}" count="{self.records}">{"".join(rows)}</pivotCacheRecords>'
        )

    def _records_xml(self, database: list[str]) -> dict[tuple[int, int], str]:
        """The `records_xml` entries by (record, database field): each replaces that record value with the
        author's element (T-4767: a value with `<tpls>`, which the builder does not write itself)."""
        raw = self.d.get("records_xml") or []
        where = f"pivot {self.name}: records_xml"
        if not isinstance(raw, list) or len(raw) > pivot_groups.MAX_LIST:
            raise ValueError(f"{where} is a list of at most {pivot_groups.MAX_LIST} entries")
        out: dict[tuple[int, int], str] = {}
        for e in raw:
            if not isinstance(e, dict) or set(e) != {"record", "field", "xml"}:
                raise ValueError(f"{where} entries are {{record, field, xml}}")
            r, name = e["record"], e["field"]
            if (
                isinstance(r, bool)
                or not isinstance(r, int)
                or not 0 <= r < self.records
                or name not in database
            ):
                raise ValueError(f"{where}: record {r!r} of field {name!r} is not in the cache")
            tag = re.match(r"<([a-z]+)", str(e["xml"]).strip())
            if tag is None or tag[1] not in _RECORD_TAGS or (r, database.index(name)) in out:
                raise ValueError(f"{where}: each entry is one of {list(_RECORD_TAGS)}, once per record value")
            out[(r, database.index(name))] = raw_element(e["xml"], tag[1], f"{where} {r} {name!r}")
        return out

    def _pivot_field(self, f: _Field) -> str:
        spec = f.spec
        attrs = _attrs(spec.get("attrs"), _MANAGED_FIELD, f"pivot {self.name} field {f.name!r} attrs")
        head = ""
        if f.axis:
            head += f' axis="{f.axis}"'
        if f.data:
            head += ' dataField="1"'
        if f.formula is not None:
            head += ' dragToRow="0" dragToCol="0" dragToPage="0"'
        subtotals = spec.get("subtotals")
        if subtotals is not None:
            if not isinstance(subtotals, list) or any(s not in SUBTOTALS for s in subtotals):
                raise ValueError(
                    f"pivot {self.name}: subtotals of {f.name!r} must be a list of {list(SUBTOTALS)}"
                )
            head += ' defaultSubtotal="0"' + "".join(f' {s}Subtotal="1"' for s in subtotals)
        if len(f.spec.get("hidden", [])) and f.axis == "axisPage":
            head += ' multipleItemSelectionAllowed="1"'
        if f.axis is None and f.labels is None:
            ext = self._field_ext(f)
            return (
                f'<pivotField{head} showAll="0"{attrs}>{ext}</pivotField>'
                if ext
                else f'<pivotField{head} showAll="0"{attrs}/>'
            )
        if f.labels is not None:
            values: list[Value] = f.labels
        else:
            values = [*f.group_labels()] if f.group is not None else f.items
        keys = {_key(v): k for k, v in enumerate(values)}
        flags: dict[int, str] = {}
        for kind, attr in (("hidden", 'h="1"'), ("collapsed", 'sd="0"')):
            for raw in spec.get(kind, []):
                k = keys.get(_key(cached_value(raw)))
                if k is None:
                    raise ValueError(f"pivot {self.name}: {kind} item {raw!r} is not in field {f.name!r}")
                flags[k] = f"{flags.get(k, '')} {attr}"
        item_attrs = spec.get("item_attrs") or {}
        if not isinstance(item_attrs, dict):
            raise ValueError(f"pivot {self.name}: item_attrs of {f.name!r} maps item texts to attributes")
        for raw, extra in item_attrs.items():
            k = keys.get(_key(cached_value(raw)))
            if k is None:
                raise ValueError(f"pivot {self.name}: item_attrs item {raw!r} is not in field {f.name!r}")
            flags[k] = flags.get(k, "") + _attrs(extra, _MANAGED_ITEM, f"pivot {self.name} item {raw!r}")
        # A calculated item is `f="1"` unless its item_attrs set `f` (Excel for the web refuses `f="1"`: T-4130).
        own_f = {
            _key(cached_value(raw))
            for raw, extra in item_attrs.items()
            if isinstance(extra, dict) and "f" in extra
        }
        for k, v in enumerate(values):
            if _key(v) in f.calculated and _key(v) not in own_f:
                flags[k] = flags.get(k, "") + ' f="1"'
        items = "".join(f'<item x="{k}"{flags.get(k, "")}/>' for k in self._item_order(f, keys, len(values)))
        tails = ["default"] if subtotals is None else list(subtotals)
        items += "".join(f'<item t="{t}"/>' for t in tails)
        ext = self._field_ext(f)
        return (
            f'<pivotField{head} showAll="0"{attrs}><items count="{len(values) + len(tails)}">{items}</items>'
            f"{ext}</pivotField>"
        )

    def _item_order(self, f: _Field, keys: dict[Any, int], count: int) -> list[int]:
        """The order of the field's `<item>` elements: the items `item_order` lists first, in that order,
        then the rest by index, or for a field a group adds, in its own order (its `item_order` attribute,
        pivot_groups.py). Unlike `order` it leaves the cache's shared items as they are, so a case can ask whether
        Excel shows the `<items>` order or the cache order (T-4164), or a group field's (T-4170)."""
        listed = f.spec.get("item_order", [])
        if not isinstance(listed, list):
            raise ValueError(f"pivot {self.name}: item_order of {f.name!r} is a list of items")
        first: list[int] = []
        for raw in listed:
            k = keys.get(_key(cached_value(raw)))
            if k is None:
                raise ValueError(f"pivot {self.name}: item_order item {raw!r} is not in field {f.name!r}")
            if k in first:
                raise ValueError(f"pivot {self.name}: item_order of {f.name!r} repeats an item")
            first.append(k)
        rest = f.item_order if f.item_order is not None else range(count)
        return first + [k for k in rest if k not in first]

    def _field_ext(self, f: _Field) -> str:
        """The field's raw children after `items` (its `xml` slots `autoSortScope` and `extLst`), in
        schema order."""
        raw = f.spec.get("xml") or {}
        if not isinstance(raw, dict) or set(raw) - {*_FIELD_SLOTS, _GROUP_SLOT}:
            raise ValueError(
                f"pivot {self.name}: the xml slots of field {f.name!r} are {[*_FIELD_SLOTS, _GROUP_SLOT]}"
            )
        return "".join(self._raw(raw, slot) for slot in _FIELD_SLOTS)

    def _group_items(self, f: _Field, group: str) -> str:
        """`group` (a cache field's `<fieldGroup>`) with the field's raw `groupItems` slot in place of the
        builder's `<groupItems>` (T-4767: group items of every type and attribute, which the builder does not
        write itself). The field must be grouped."""
        raw = f.spec.get("xml") or {}
        if not isinstance(raw, dict) or _GROUP_SLOT not in raw:
            return group
        if "<groupItems" not in group:
            raise ValueError(f"pivot {self.name}: field {f.name!r} has no group for its xml groupItems")
        start, end = group.index("<groupItems"), group.index("</groupItems>") + len("</groupItems>")
        return group[:start] + self._raw(raw, _GROUP_SLOT) + group[end:]

    def _page_item(self, i: int, item: Value | None, extra: str) -> str:
        if item is None:
            return f'<pageField fld="{i}" hier="-1"{extra}/>'
        k = [_key(v) for v in self.fields[i].items].index(_key(item))
        return f'<pageField fld="{i}" item="{k}" hier="-1"{extra}/>'

    def table_definition(self, cache_id: int, top: int, left: int) -> str:
        table = self.d.get("table") or {}
        extra = _attrs(table, _MANAGED_TABLE, f"pivot {self.name} table")
        head = "".join(f' {k}="{v}"' for k, v in _TABLE_DEFAULTS.items() if k not in table)
        if (
            -2 in self.rows
        ):  # the Values field on rows: Excel for the web refuses the file without it (T-4138)
            head += ' dataOnRows="1"'
        raw = self.d.get("xml") or {}
        if not isinstance(raw, dict) or (unknown := set(raw) - {*_SLOTS, *_AXIS_SLOTS}):
            raise ValueError(
                f"pivot {self.name}: xml slots are {[*_AXIS_SLOTS, *_SLOTS]}, not {sorted(unknown or [])}"
            )
        parts = [
            f'<location ref="{self._area(top, left)}" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"'
            + (f' rowPageCount="{len(self.pages)}" colPageCount="1"' if self.pages else "")
            + "/>",
            f'<pivotFields count="{len(self.fields)}">{"".join(self._pivot_field(f) for f in self.fields)}</pivotFields>',
        ]
        for tag, axis, lines in (
            ("rowFields", self.rows, "rowItems"),
            ("colFields", self.columns, "colItems"),
        ):
            if axis:
                fields = "".join(f'<field x="{x}"/>' for x in axis)
                parts.append(f'<{tag} count="{len(axis)}">{fields}</{tag}>')
            parts.append(self._raw(raw, lines))
        if self.pages:
            pages = "".join(self._page_item(i, item, extra) for i, item, extra in self.pages)
            parts.append(f'<pageFields count="{len(self.pages)}">{pages}</pageFields>')
        data = "".join(self._data_field(e) for e in self.data)
        parts.append(f'<dataFields count="{len(self.data)}">{data}</dataFields>')
        for slot in _SLOTS[:4]:
            parts.append(self._raw(raw, slot))
        parts.append(
            '<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1" showColHeaders="1"'
            ' showRowStripes="0" showColStripes="0" showLastColumn="1"/>'
        )
        parts.extend(self._raw(raw, slot) for slot in _SLOTS[4:])
        return (
            f'<pivotTableDefinition xmlns="{MAIN_NS}" name={quoteattr(self.name)} cacheId="{cache_id}"{head}{extra}>'
            f"{''.join(parts)}</pivotTableDefinition>"
        )

    def _area(self, top: int, left: int) -> str:
        """The `<location>` ref: `area` when given (starting at the location), else the anchor and the
        cell diagonally below it, which Excel replaces when it renders."""
        area = self.d.get("area")
        if area is None:
            corner = xlsxwriter.utility.xl_rowcol_to_cell(top + 1, left + 1)
            return f"{xlsxwriter.utility.xl_rowcol_to_cell(top, left)}:{corner}"
        bad = ValueError(f"pivot {self.name}: area {area!r} is a range A1:B2 starting at the location")
        if not isinstance(area, str) or not _AREA.fullmatch(area):
            raise bad
        (r0, c0), (r1, c1) = (xlsxwriter.utility.xl_cell_to_rowcol(e) for e in area.split(":"))
        if (r0, c0) != (top, left) or r1 < r0 or c1 < c0:
            raise bad
        return area

    def _data_field(self, e: dict[str, Any]) -> str:
        """One `<dataField>` (sum, `showDataAs`, `numFmtId` written only when given). An x14 mode is
        written as the extension's `pivotShowAs` instead of `showDataAs`."""
        attrs = f' name={quoteattr(e["name"])} fld="{e["fld"]}"'
        if e["subtotal"] != "sum":
            attrs += f' subtotal="{e["subtotal"]}"'
        show = e.get("showDataAs")
        ext = ""
        if show in SHOW_AS_X14:
            ext = (
                f'<extLst><ext uri="{X14_DATA_FIELD_URI}" xmlns:x14="{X14_NS}">'
                f'<x14:dataField pivotShowAs="{show}"/></ext></extLst>'
            )
        elif show is not None:
            attrs += f' showDataAs="{show}"'
        base_field = e.get("baseField", 0)
        base = self._index(base_field) if isinstance(base_field, str) else int(base_field)
        attrs += f' baseField="{base}" baseItem="{int(e.get("baseItem", 0))}"'
        if "numFmtId" in e:
            attrs += f' numFmtId="{int(e["numFmtId"])}"'
        return f"<dataField{attrs}>{ext}</dataField>" if ext else f"<dataField{attrs}/>"

    def _raw(self, raw: dict[str, Any], slot: str, tag: str | None = None) -> str:
        """The author's XML for `slot`: one `<slot>` (or `<tag>`) element (`raw_element`)."""
        if slot not in raw:
            return ""
        return raw_element(raw[slot], tag or slot, f"pivot {self.name}: xml slot {slot}")


def raw_element(raw: Any, tag: str, where: str) -> str:
    """Raw XML an author gives: one `<tag>` element, no declarations, DTDs or entities. It is not parsed
    here (a malformed one is Excel's refusal to see, as `OpenWorkbookBlockedWorkbook`)."""
    text = str(raw).strip()
    whole = re.fullmatch(rf"<{tag}(?:\s[^<>]*)?(?:/>|>.*</{tag}>)", text, flags=re.DOTALL)
    bare_amp = "&" in re.sub(r"&(?:amp|lt|gt|quot|apos);", "", text)
    if whole is None or "<!" in text or "<?" in text or bare_amp:
        raise ValueError(f"{where} must be one <{tag}> element without declarations")
    return text
