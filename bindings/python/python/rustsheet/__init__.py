"""IronCalc: create, edit and evaluate Excel spreadsheets.

There are two APIs:

* The **facade API** (:class:`Workbook`): the typed public API. Excel errors are returned as
  :class:`XlError` values, edits are atomic transactions (``with wb.edit() as tx``) and every edit
  reports a change set in ``wb.changes``.
* The **user API** (:class:`UserModel`): the same high level API used by the
  IronCalc web application. Every action evaluates the workbook, keeps
  undo/redo history and produces diffs for collaboration.
* The **raw API** (:class:`Model`): a low level API. Nothing is evaluated
  automatically (call :meth:`Model.evaluate` yourself), there is no undo/redo
  and no diffs. It is faster and more flexible, but easier to get wrong.

Quick start::

    import rustsheet as rs

    wb = rs.Workbook()
    with wb.edit() as tx:
        tx["Sheet1!A1"] = 10
        tx["Sheet1!A2"] = rs.Formula("=A1*2")
    print(wb["Sheet1!A2"])  # 20.0
    print(wb.changes["values"])  # [{"ref": "Sheet1!A2", "before": None, "after": 20.0}]
    wb.save_xlsx("my-workbook.xlsx")
"""

from rustsheet._rustsheet import (
    CellType,
    EditError,
    Formula,
    LoadError,
    Model,
    RefError,
    RustSheetError,
    SaveError,
    Transaction,
    UserModel,
    Workbook,
    WorkbookError,
    XlError,
    __version__,
    column_name_from_number,
    column_number_from_name,
    create,
    create_user_model,
    create_user_model_from_bytes,
    create_user_model_from_icalc,
    create_user_model_from_xlsx,
    get_all_timezones,
    get_supported_locales,
    load_from_bytes,
    load_from_icalc,
    load_from_xlsx,
    quote_name,
    test_panic,
)


def open(path: str, recalc: bool = True) -> Workbook:
    """Open an xlsx file as a :class:`Workbook` (shorthand for ``Workbook.open``)."""
    return Workbook.open(path, recalc)


__all__ = [
    "CellType",
    "EditError",
    "Formula",
    "LoadError",
    "Model",
    "RefError",
    "RustSheetError",
    "SaveError",
    "Transaction",
    "UserModel",
    "Workbook",
    "WorkbookError",
    "XlError",
    "open",
    "__version__",
    "column_name_from_number",
    "column_number_from_name",
    "create",
    "create_user_model",
    "create_user_model_from_bytes",
    "create_user_model_from_icalc",
    "create_user_model_from_xlsx",
    "get_all_timezones",
    "get_supported_locales",
    "load_from_bytes",
    "load_from_icalc",
    "load_from_xlsx",
    "quote_name",
    "test_panic",
]
