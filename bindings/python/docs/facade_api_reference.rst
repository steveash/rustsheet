Facade API
==========

The typed public API (``rustsheet.Workbook``). Values are ``None | float | str | bool |
XlError``; Excel error values are returned, never raised. ``RefError``, ``EditError``,
``LoadError`` and ``SaveError`` subclass ``RustSheetError``. Loading, saving and recalculation
release the GIL.

.. literalinclude:: examples/facade.py
   :language: python

.. autoclass:: rustsheet.Workbook
   :members:

.. autoclass:: rustsheet.Transaction
   :members:

.. autoclass:: rustsheet.XlError
   :members:

.. autoclass:: rustsheet.Formula
   :members:
