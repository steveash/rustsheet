#!/usr/bin/env python3
"""The one GitHub client of the merge scripts (SPEC-0027 section 4, decision 0148).

`Client` builds every request the merge lock, the readiness handoff and the merge train make;
`GhApi` sends them with `gh api` (REST only: the session proxy refuses GraphQL), and `FakeGh`
answers them from a table for tests. Both subclasses share the request building, so a test of the
fake exercises the same request shapes the real client sends (scripts/test_gh_client.py).

Every value that lands in a URL is validated first (PR numbers, shas, logins, labels, workflow
file names, branch names), so no caller-supplied text can reach another API path. Lists are read
with `page=` on `repos/{owner}/{repo}` URLs, never by following `Link` headers: the proxy answers
HTTP 403 to their numeric repository ids (verified 2026-10-28).
"""

from __future__ import annotations

import json
import re
import subprocess
import urllib.parse
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# `gh api` gets this long per request; a merge is the slowest call and takes a few seconds.
TIMEOUT_SECONDS = 30
PER_PAGE = 100
# 50 pages of 100 bounds any list read (open PRs, a PR's comments, a commit's check runs).
MAX_PAGES = 50

SHA = re.compile(r"[0-9a-f]{40}")
# An allowlist for logins in a URL (GitHub's own rule is stricter): alphanumerics and hyphens, at most
# 39 characters, and `[bot]` apps.
LOGIN = re.compile(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,38})(?:\[bot\])?")
LABEL = re.compile(r"[A-Za-z0-9][A-Za-z0-9 ._-]{0,49}")
WORKFLOW = re.compile(r"[A-Za-z0-9_.-]{1,64}\.ya?ml")
BRANCH = re.compile(r"[A-Za-z0-9][A-Za-z0-9._/-]{0,199}")
# Owner, repository and input names; a name of dots only (`..`) is refused, it would change the path.
NAME = re.compile(r"(?!\.+$)[A-Za-z0-9_.-]{1,100}")
REMOTE_URL = re.compile(r"github\.com[:/]+([^/\s]+)/([^/\s]+?)(?:\.git)?/?$")
MERGE_METHODS = ("squash", "merge", "rebase")


class GhError(Exception):
    """A GitHub request failed, or an argument was refused before it was sent."""


@dataclass(frozen=True)
class Request:
    """One REST call: `method` on `path` (relative to the API root), `query` pairs, a JSON body."""

    method: str
    path: str
    query: tuple[tuple[str, str], ...] = ()
    body: Mapping[str, object] | None = None

    def url(self) -> str:
        """The path with its query string, as `gh api` takes it."""
        return f"{self.path}?{urllib.parse.urlencode(self.query)}" if self.query else self.path


def checked(pattern: re.Pattern[str], value: str, what: str) -> str:
    """`value` if it is ASCII and matches `pattern` in full; GhError otherwise."""
    if not (isinstance(value, str) and value.isascii() and pattern.fullmatch(value)):
        raise GhError(f"refused {what}: {value!r}")
    return value


def number(value: int | str, what: str = "PR number") -> int:
    """`value` as a positive integer below 10^9 (ASCII digits only); GhError otherwise."""
    text = str(value)
    if not (text.isascii() and text.isdigit() and len(text) <= 9) or int(text) == 0:
        raise GhError(f"refused {what}: {value!r}")
    return int(text)


def as_dict(value: object) -> dict[str, object]:
    """`value` if it is a JSON object; GhError otherwise."""
    if not isinstance(value, dict):
        raise GhError(f"expected a JSON object, got {type(value).__name__}")
    return value


class Client:
    """The operations of SPEC-0027 section 4 on one repository; subclasses implement `send`."""

    def __init__(self, owner: str, repo: str) -> None:
        self.base = f"repos/{checked(NAME, owner, 'owner')}/{checked(NAME, repo, 'repository')}"

    def send(self, request: Request) -> object:
        raise NotImplementedError

    def call(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]] = (),
        body: Mapping[str, object] | None = None,
    ) -> object:
        return self.send(Request(method, f"{self.base}/{path}", tuple(query), body))

    def pages(self, path: str, query: Sequence[tuple[str, str]] = (), key: str | None = None) -> list[object]:
        """Every item of a list endpoint, read with `page=` (see the module docstring)."""
        items: list[object] = []
        for page in range(1, MAX_PAGES + 1):
            reply = self.call("GET", path, [*query, ("per_page", str(PER_PAGE)), ("page", str(page))])
            batch = as_dict(reply).get(key) if key else reply
            if not isinstance(batch, list):
                raise GhError(f"expected a list from {path}")
            items.extend(batch)
            if len(batch) < PER_PAGE:
                return items
        raise GhError(f"{path} has more than {MAX_PAGES * PER_PAGE} items")

    # Pull requests and comments.
    def pull(self, pr: int | str) -> dict[str, object]:
        return as_dict(self.call("GET", f"pulls/{number(pr)}"))

    def pr_finished(self, pr: int | str) -> bool | None:
        """Whether PR `pr` is merged or closed; None when the read fails (decision 0097)."""
        try:
            state = self.pull(pr)
        except GhError:
            return None
        if "state" not in state:
            return None
        return bool(state.get("merged")) or state["state"] != "open"

    def pulls(self, state: str = "open") -> list[object]:
        if state not in ("open", "closed", "all"):
            raise GhError(f"refused PR state: {state!r}")
        return self.pages("pulls", [("state", state)])

    def comments(self, pr: int | str) -> list[object]:
        """The PR's conversation comments (issue comments), oldest first."""
        return self.pages(f"issues/{number(pr)}/comments")

    def permission(self, login: str) -> str:
        """`login`'s permission on the repository: admin, maintain, write, triage, read or none."""
        reply = as_dict(self.call("GET", f"collaborators/{checked(LOGIN, login, 'login')}/permission"))
        value = reply.get("permission")
        if not isinstance(value, str):
            raise GhError(f"no permission for {login}")
        return value

    def comment(self, pr: int | str, body: str) -> dict[str, object]:
        if not body or len(body) > 65536:
            raise GhError("refused comment: empty or over 65536 characters")
        return as_dict(self.call("POST", f"issues/{number(pr)}/comments", body={"body": body}))

    # Labels.
    def add_label(self, pr: int | str, label: str) -> object:
        return self.call(
            "POST", f"issues/{number(pr)}/labels", body={"labels": [checked(LABEL, label, "label")]}
        )

    def remove_label(self, pr: int | str, label: str) -> object:
        name = urllib.parse.quote(checked(LABEL, label, "label"), safe="")
        return self.call("DELETE", f"issues/{number(pr)}/labels/{name}")

    # CI.
    def check_runs(self, sha: str) -> list[object]:
        return self.pages(f"commits/{checked(SHA, sha, 'sha')}/check-runs", key="check_runs")

    def workflow_runs(self, *, branch: str | None = None, head_sha: str | None = None) -> list[object]:
        query = []
        if branch is not None:
            query.append(("branch", checked(BRANCH, branch, "branch")))
        if head_sha is not None:
            query.append(("head_sha", checked(SHA, head_sha, "sha")))
        return self.pages("actions/runs", query, key="workflow_runs")

    def dispatch(self, workflow: str, ref: str, inputs: Mapping[str, str] | None = None) -> object:
        body: dict[str, object] = {"ref": checked(BRANCH, ref, "ref")}
        if inputs:
            body["inputs"] = {checked(NAME, k, "input"): str(v) for k, v in inputs.items()}
        return self.call(
            "POST", f"actions/workflows/{checked(WORKFLOW, workflow, 'workflow')}/dispatches", body=body
        )

    # Git data and merging.
    def commit(self, sha: str) -> dict[str, object]:
        return as_dict(self.call("GET", f"git/commits/{checked(SHA, sha, 'sha')}"))

    def merge(
        self, pr: int | str, head_sha: str, *, title: str, message: str = "", method: str = "squash"
    ) -> dict[str, object]:
        """Merge PR `pr` only while its head is `head_sha` (the REST `sha` field): GitHub answers
        409 when the head moved, so a merge never lands an untested head."""
        if method not in MERGE_METHODS:
            raise GhError(f"refused merge method: {method!r}")
        if not title or "\n" in title:
            raise GhError("refused merge title: empty or multi-line")
        body = {
            "merge_method": method,
            "sha": checked(SHA, head_sha, "sha"),
            "commit_title": title,
            "commit_message": message,
        }
        return as_dict(self.call("PUT", f"pulls/{number(pr)}/merge", body=body))


class GhApi(Client):
    """Sends requests with `gh api`, which carries the session's or CI's GitHub credentials."""

    @staticmethod
    def argv(request: Request) -> list[str]:
        command = [
            "gh",
            "api",
            "--method",
            request.method,
            "-H",
            "Accept: application/vnd.github+json",
            request.url(),
        ]
        return command + ["--input", "-"] if request.body is not None else command

    def send(self, request: Request) -> object:
        stdin = json.dumps(request.body) if request.body is not None else ""
        try:
            result = subprocess.run(
                self.argv(request),
                input=stdin,
                capture_output=True,
                text=True,
                timeout=TIMEOUT_SECONDS,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise GhError(f"gh api {request.method} {request.path}: {error.__class__.__name__}") from error
        if result.returncode != 0:
            detail = (result.stderr.strip().splitlines() or ["no output"])[0][:200]
            raise GhError(f"gh api {request.method} {request.path}: {detail}")
        try:
            return json.loads(result.stdout) if result.stdout.strip() else None
        except ValueError as error:
            raise GhError(f"gh api {request.method} {request.path}: not JSON") from error


@dataclass
class Reply:
    """A canned answer of `FakeGh`: a JSON value, or the error to raise."""

    value: object = None
    error: str | None = None


class FakeGh(Client):
    """Answers requests from `replies`, keyed by (method, url); records every request it gets."""

    def __init__(
        self, owner: str = "o", repo: str = "r", replies: Mapping[tuple[str, str], Reply] | None = None
    ) -> None:
        super().__init__(owner, repo)
        self.replies = dict(replies or {})
        self.requests: list[Request] = []

    def send(self, request: Request) -> object:
        self.requests.append(request)
        reply = self.replies.get((request.method, request.url()))
        if reply is None:
            raise GhError(f"fake: no reply for {request.method} {request.url()}")
        if reply.error is not None:
            raise GhError(reply.error)
        return reply.value


def owner_repo(url: str) -> tuple[str, str] | None:
    """The (owner, repository) of a github.com remote URL, or None."""
    match = REMOTE_URL.search(url)
    return (match[1], match[2]) if match else None


def from_remote(root: Path = ROOT, remote: str = "origin") -> GhApi | None:
    """The real client for `remote`'s repository, or None when it is not a github.com remote."""
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "remote", "get-url", remote],
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=TIMEOUT_SECONDS,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    pair = owner_repo(result.stdout.strip())
    if pair is None:
        return None
    try:
        return GhApi(*pair)
    except GhError:
        return None
