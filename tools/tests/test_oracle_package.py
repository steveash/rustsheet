"""`patch_parts`: ordered patches per part, ZipInfo kept, missing parts refused."""

from __future__ import annotations

import io
import zipfile

import pytest

from rustsheet_tools.oracle.package import PackageError, patch_parts


def _zip() -> bytes:
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as zf:
        zf.writestr(zipfile.ZipInfo("a.xml", (2000, 1, 1, 0, 0, 0)), "x")
        zf.writestr(zipfile.ZipInfo("b.xml", (2000, 1, 1, 0, 0, 0)), "keep")
    return out.getvalue()


def test_patches_apply_in_order_and_keep_zipinfo() -> None:
    patched = patch_parts(_zip(), [("a.xml", lambda t: t + "y"), ("a.xml", lambda t: t + "z")])
    with zipfile.ZipFile(io.BytesIO(patched)) as zf:
        assert zf.read("a.xml") == b"xyz"
        assert zf.read("b.xml") == b"keep"
        assert zf.getinfo("a.xml").date_time == (2000, 1, 1, 0, 0, 0)


def test_missing_part_is_a_package_error() -> None:
    with pytest.raises(PackageError, match="not in the package"):
        patch_parts(_zip(), [("c.xml", lambda t: t)])
