"""Oracle backends: something that takes xlsx bytes and returns the same workbook
fully recalculated by a reference implementation, as xlsx bytes.

Every backend returns a *recalculated xlsx*; values are then extracted with the single
Rust snapshot parser. That keeps one value-decoding path for corpora, Graph, COM and
LibreOffice alike, and preserves full 17-digit precision.

Backends:
  excel-graph     Microsoft Graph (Excel for the web calc service). Needs a work/school tenant.
  excel-desktop   Excel for Windows via COM (pywin32). Run on the Windows oracle box.
  libreoffice     soffice --headless. APPROXIMATE: triage only, never a golden.
"""

from __future__ import annotations

import contextlib
import io
import os
import shutil
import subprocess
import tempfile
import time
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol
from urllib.parse import quote

import requests


class OracleError(RuntimeError):
    pass


@dataclass(frozen=True)
class OracleInfo:
    kind: str
    application: str
    version: str | None = None


class Backend(Protocol):
    name: str

    def available(self) -> tuple[bool, str]:
        """(usable?, human-readable reason)."""
        ...

    def recalc(self, xlsx: bytes) -> tuple[bytes, OracleInfo]:
        """Fully recalculates the workbook and returns the saved bytes."""
        ...


def force_full_calc_on_load(xlsx: bytes) -> bytes:
    """Sets `<calcPr fullCalcOnLoad="1"/>` so any Excel opening the file recalculates everything."""
    src = zipfile.ZipFile(io.BytesIO(xlsx))
    out_buf = io.BytesIO()
    with zipfile.ZipFile(out_buf, "w", zipfile.ZIP_DEFLATED) as out:
        for item in src.infolist():
            data = src.read(item.filename)
            if item.filename == "xl/workbook.xml":
                text = data.decode("utf-8")
                if "<calcPr" in text:
                    if "fullCalcOnLoad" not in text:
                        text = text.replace("<calcPr", '<calcPr fullCalcOnLoad="1"', 1)
                else:
                    text = text.replace("</workbook>", '<calcPr fullCalcOnLoad="1"/></workbook>', 1)
                data = text.encode("utf-8")
            out.writestr(item, data)
    return out_buf.getvalue()


# --------------------------------------------------------------------------- Graph


GRAPH = "https://graph.microsoft.com/v1.0"
SCOPES = ["Files.ReadWrite"]


def _cache_path() -> Path:
    override = os.environ.get("RUSTSHEET_ORACLE_MSAL_CACHE")
    if override:
        return Path(override)
    return Path.home() / ".config" / "rustsheet-oracle" / "msal_cache.json"


class GraphBackend:
    """Excel for the web via Microsoft Graph.

    Environment:
      RUSTSHEET_ORACLE_TENANT_ID, RUSTSHEET_ORACLE_CLIENT_ID      (required)
      RUSTSHEET_ORACLE_CLIENT_SECRET + RUSTSHEET_ORACLE_USER        (app-only mode; USER is the
          work email whose OneDrive holds the scratch files; or RUSTSHEET_ORACLE_DRIVE_ID instead)
      RUSTSHEET_ORACLE_MSAL_CACHE_B64                               (delegated bootstrap cache)
      RUSTSHEET_ORACLE_FOLDER (default /rustsheet-oracle)
    """

    name = "excel-graph"

    def __init__(self) -> None:
        self.tenant = os.environ.get("RUSTSHEET_ORACLE_TENANT_ID", "")
        self.client_id = os.environ.get("RUSTSHEET_ORACLE_CLIENT_ID", "")
        self.secret = os.environ.get("RUSTSHEET_ORACLE_CLIENT_SECRET")
        self.drive_id = os.environ.get("RUSTSHEET_ORACLE_DRIVE_ID")
        self.user = os.environ.get("RUSTSHEET_ORACLE_USER")
        self.folder = os.environ.get("RUSTSHEET_ORACLE_FOLDER", "/rustsheet-oracle").strip("/")
        self._session = requests.Session()

    def available(self) -> tuple[bool, str]:
        if not (self.tenant and self.client_id):
            return False, "set RUSTSHEET_ORACLE_TENANT_ID and RUSTSHEET_ORACLE_CLIENT_ID"
        if self.secret and not (self.drive_id or self.user):
            return (
                False,
                "app-only mode needs RUSTSHEET_ORACLE_USER (work email) or RUSTSHEET_ORACLE_DRIVE_ID",
            )
        if not self.secret and not (
            _cache_path().exists() or os.environ.get("RUSTSHEET_ORACLE_MSAL_CACHE_B64")
        ):
            return False, "delegated mode needs a token cache: run `rustsheet-oracle login`"
        return True, "app-only" if self.secret else "delegated"

    # -- auth
    def _token(self) -> str:
        import base64  # noqa: PLC0415

        import msal  # noqa: PLC0415

        authority = f"https://login.microsoftonline.com/{self.tenant}"
        if self.secret:
            app = msal.ConfidentialClientApplication(
                self.client_id, authority=authority, client_credential=self.secret
            )
            result = app.acquire_token_for_client(scopes=["https://graph.microsoft.com/.default"])
        else:
            cache = msal.SerializableTokenCache()
            path = _cache_path()
            if path.exists():
                cache.deserialize(path.read_text())
            elif os.environ.get("RUSTSHEET_ORACLE_MSAL_CACHE_B64"):
                cache.deserialize(base64.b64decode(os.environ["RUSTSHEET_ORACLE_MSAL_CACHE_B64"]).decode())
            app = msal.PublicClientApplication(self.client_id, authority=authority, token_cache=cache)
            accounts = app.get_accounts()
            if not accounts:
                raise OracleError("no cached account; run `rustsheet-oracle login`")
            result = app.acquire_token_silent(SCOPES, account=accounts[0])
            if cache.has_state_changed:  # refresh tokens rotate: always write back
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(cache.serialize())
                path.chmod(0o600)
        if not result or "access_token" not in result:
            raise OracleError(f"token acquisition failed: {result}")
        token: str = result["access_token"]
        return token

    def login(self) -> None:
        """Interactive device-code login (delegated mode). Run once on the oracle machine."""
        import msal  # noqa: PLC0415

        cache = msal.SerializableTokenCache()
        app = msal.PublicClientApplication(
            self.client_id, authority=f"https://login.microsoftonline.com/{self.tenant}", token_cache=cache
        )
        flow = app.initiate_device_flow(scopes=SCOPES)
        print(flow["message"])
        result = app.acquire_token_by_device_flow(flow)
        if "access_token" not in result:
            raise OracleError(f"login failed: {result}")
        path = _cache_path()
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(cache.serialize())
        path.chmod(0o600)
        print(f"token cache written to {path}")

    # -- http
    def _drive(self) -> str:
        if self.drive_id:
            return f"{GRAPH}/drives/{self.drive_id}"
        if self.user:
            return f"{GRAPH}/users/{quote(self.user)}/drive"
        return f"{GRAPH}/me/drive"

    def _call(
        self,
        method: str,
        url: str,
        *,
        headers: dict[str, str] | None = None,
        json: Any = None,
        data: bytes | None = None,
    ) -> requests.Response:
        for attempt in range(8):
            all_headers = {"Authorization": f"Bearer {self._token()}", **(headers or {})}
            resp = self._session.request(method, url, headers=all_headers, json=json, data=data, timeout=120)
            if resp.status_code in (429, 503, 504):
                time.sleep(float(resp.headers.get("Retry-After", 2**attempt)))
                continue
            if resp.status_code >= 400:
                raise OracleError(f"{method} {url} -> {resp.status_code}: {resp.text[:500]}")
            return resp
        raise OracleError(f"{method} {url}: retries exhausted")

    def recalc(self, xlsx: bytes) -> tuple[bytes, OracleInfo]:
        import hashlib  # noqa: PLC0415

        name = hashlib.sha256(xlsx).hexdigest() + ".xlsx"
        drive = self._drive()
        item = self._call(
            "PUT", f"{drive}/root:/{self.folder}/{name}:/content", data=force_full_calc_on_load(xlsx)
        ).json()
        item_url = f"{drive}/items/{item['id']}"
        try:
            session = self._call(
                "POST", f"{item_url}/workbook/createSession", json={"persistChanges": True}
            ).json()
            sid = {"workbook-session-id": session["id"]}
            self._call(
                "POST",
                f"{item_url}/workbook/application/calculate",
                json={"calculationType": "FullRebuild"},
                headers=sid,
            )
            self._call("POST", f"{item_url}/workbook/closeSession", headers=sid)
            # Persisting is asynchronous: poll until the eTag moves, then download.
            etag = item.get("eTag")
            for _ in range(30):
                meta = self._call("GET", item_url).json()
                if meta.get("eTag") != etag:
                    break
                time.sleep(2)
            content = self._call("GET", f"{item_url}/content").content
        finally:
            with contextlib.suppress(OracleError):
                self._call("DELETE", item_url)
        return content, OracleInfo(kind=self.name, application="Excel for the web (Graph)")


# --------------------------------------------------------------------------- Windows COM


class ExcelDesktopBackend:
    """Excel for Windows via COM. Must run in an interactive desktop session on Windows."""

    name = "excel-desktop"

    def available(self) -> tuple[bool, str]:
        if os.name != "nt":
            return False, "requires Windows with Excel installed (see docs/architecture/oracle.md)"
        try:
            import win32com.client  # noqa: F401, PLC0415
        except ImportError:
            return False, "pip install 'rustsheet-tools[windows]' (pywin32)"
        return True, "COM"

    def recalc(self, xlsx: bytes) -> tuple[bytes, OracleInfo]:
        import win32com.client  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp) / "in.xlsx"
            dst = Path(tmp) / "out.xlsx"
            src.write_bytes(force_full_calc_on_load(xlsx))
            excel = win32com.client.DispatchEx("Excel.Application")
            try:
                excel.Visible = False
                excel.DisplayAlerts = False
                excel.ScreenUpdating = False
                excel.EnableEvents = False
                excel.AskToUpdateLinks = False
                excel.AutomationSecurity = 3  # msoAutomationSecurityForceDisable
                wb = excel.Workbooks.Open(str(src), UpdateLinks=0, ReadOnly=True, AddToMru=False)
                excel.CalculateFullRebuild()
                wb.SaveAs(str(dst), FileFormat=51)
                wb.Close(SaveChanges=False)
                version = str(excel.Version) + "." + str(excel.Build)
            finally:
                excel.Quit()
            return dst.read_bytes(), OracleInfo(
                kind=self.name, application="Microsoft Excel", version=version
            )


# --------------------------------------------------------------------------- LibreOffice


class LibreOfficeBackend:
    """LibreOffice headless. APPROXIMATE — results are labelled and never used as goldens."""

    name = "libreoffice"

    def available(self) -> tuple[bool, str]:
        soffice = shutil.which("soffice")
        if soffice is None:
            return False, "install LibreOffice (apt install libreoffice-calc)"
        program = Path(soffice).resolve().parent
        if not any(program.glob("libsclo.so*")) and not (program / "sclo.dll").exists():
            return False, "LibreOffice Calc component missing (apt install libreoffice-calc)"
        return True, "soffice + Calc"

    def recalc(self, xlsx: bytes) -> tuple[bytes, OracleInfo]:
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp) / "in.xlsx"
            outdir = Path(tmp) / "out"
            src.write_bytes(force_full_calc_on_load(xlsx))
            profile = Path(tmp) / "profile"
            subprocess.run(
                [
                    "soffice",
                    f"-env:UserInstallation=file://{profile}",
                    "--headless",
                    "--calc",
                    "--convert-to",
                    "xlsx:Calc MS Excel 2007 XML",
                    "--outdir",
                    str(outdir),
                    str(src),
                ],
                check=True,
                capture_output=True,
                timeout=300,
            )
            return (outdir / "in.xlsx").read_bytes(), OracleInfo(
                kind=self.name, application="LibreOffice (approximate)"
            )


BACKENDS: dict[str, type[GraphBackend] | type[ExcelDesktopBackend] | type[LibreOfficeBackend]] = {
    "excel-graph": GraphBackend,
    "excel-desktop": ExcelDesktopBackend,
    "libreoffice": LibreOfficeBackend,
}
