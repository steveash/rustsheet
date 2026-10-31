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
from collections.abc import Callable, Mapping, Sequence
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
# A label's colour: six hex digits, no `#` (the REST labels endpoint's form).
COLOR = re.compile(r"[0-9a-f]{6}")
# The activity endpoint's paging cursor (its `after=` value): base64url text.
CURSOR = re.compile(r"[A-Za-z0-9_=-]{1,200}")
# A run list's `created` filter: one ISO-8601 UTC instant, as `>=` it or a `start..end` range.
CREATED = re.compile(r"(?:>=)?\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ(?:\.\.\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ)?")
EVENT = re.compile(r"[a-z_]{1,40}")
# Workflow run ids passed 10^10 in 2026, so they get their own bound, not `number`'s.
RUN_ID = re.compile(r"[1-9][0-9]{0,19}")
NEXT_AFTER = re.compile(r'<[^>]*[?&]after=([^&>]+)[^>]*>;\s*rel="next"')
# How `gh api` reports a missing object.
NOT_FOUND = "HTTP 404"
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


def branch_name(name: str) -> str:
    """A branch name safe in a URL path: `BRANCH`, and no empty, `.`-led or `.lock` segment, so
    no `..` can climb out of `git/refs/heads/`."""
    checked(BRANCH, name, "branch")
    if any(not part or part.startswith(".") or part.endswith(".lock") for part in name.split("/")):
        raise GhError(f"refused branch: {name!r}")
    return name


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

    def send_cursor(self, request: Request) -> tuple[object, str | None]:
        """`send`, plus the cursor of the next page (the `after=` of a `rel="next"` link), if any."""
        raise NotImplementedError

    def call(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]] = (),
        body: Mapping[str, object] | None = None,
    ) -> object:
        return self.send(Request(method, f"{self.base}/{path}", tuple(query), body))

    def pages(
        self,
        path: str,
        query: Sequence[tuple[str, str]] = (),
        key: str | None = None,
        enough: Callable[[list[object]], bool] | None = None,
    ) -> list[object]:
        """Every item of a list endpoint, read with `page=` (see the module docstring). `enough`
        sees each page and ends the read early when it returns True (for lists sorted newest first)."""
        items: list[object] = []
        for page in range(1, MAX_PAGES + 1):
            reply = self.call("GET", path, [*query, ("per_page", str(PER_PAGE)), ("page", str(page))])
            batch = as_dict(reply).get(key) if key else reply
            if not isinstance(batch, list):
                raise GhError(f"expected a list from {path}")
            items.extend(batch)
            if len(batch) < PER_PAGE or (enough is not None and enough(batch)):
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

    def updated_pulls(self, since: str) -> list[object]:
        """Every PR, newest update first, down to the first page that reaches back past `since`
        (an ISO-8601 UTC instant, which compares as text)."""
        def enough(batch: list[object]) -> bool:
            last = batch[-1] if batch else None
            return isinstance(last, dict) and str(last.get("updated_at", "")) < since

        query = [("state", "all"), ("sort", "updated"), ("direction", "desc")]
        return self.pages("pulls", query, enough=enough)

    def closed_labelled(self, label: str) -> list[object]:
        """Closed issues and PRs that still carry `label` (a PR has a `pull_request` key). The
        issues endpoint filters by label, so the listing is only the label's leftovers, never every
        closed PR."""
        return self.pages("issues", [("state", "closed"), ("labels", checked(LABEL, label, "label"))])

    def pull_files(self, pr: int | str) -> list[object]:
        """The files the PR changes against its merge base, as GitHub lists them (renames as
        `renamed`; the head blob sha of each file in `sha`)."""
        return self.pages(f"pulls/{number(pr)}/files")

    def comments(self, pr: int | str) -> list[object]:
        """The PR's conversation comments (issue comments), oldest first."""
        return self.pages(f"issues/{number(pr)}/comments")

    def pull_commits(self, pr: int | str) -> list[object]:
        """The PR's commits, oldest first (their messages carry the squash trailers)."""
        return self.pages(f"pulls/{number(pr)}/commits")

    def reviews(self, pr: int | str) -> list[object]:
        """The PR's reviews, oldest first."""
        return self.pages(f"pulls/{number(pr)}/reviews")

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

    def create_pull(self, title: str, head: str, body: str, base: str = "main") -> dict[str, object]:
        """Opens a PR from the same-repository branch `head` into `base`."""
        if not title or "\n" in title or len(title) > 256:
            raise GhError("refused PR title: empty, multi-line or over 256 characters")
        if not body or len(body) > 65536:
            raise GhError("refused PR body: empty or over 65536 characters")
        payload = {"title": title, "head": branch_name(head), "base": branch_name(base), "body": body}
        return as_dict(self.call("POST", "pulls", body=payload))

    def set_base(self, pr: int | str, base: str) -> dict[str, object]:
        """Sets PR `pr`'s base branch to `base`. Setting it to the base it already has is how the
        merge train makes GitHub recompute a stale test merge (decision 0200)."""
        return as_dict(self.call("PATCH", f"pulls/{number(pr)}", body={"base": branch_name(base)}))

    def delete_branch(self, name: str) -> object:
        """Deletes the branch `name` (never `main`)."""
        if branch_name(name) == "main":
            raise GhError("refused branch deletion: main")
        return self.call("DELETE", f"git/refs/heads/{name}")

    # Labels.
    def add_label(self, pr: int | str, label: str) -> object:
        return self.call(
            "POST", f"issues/{number(pr)}/labels", body={"labels": [checked(LABEL, label, "label")]}
        )

    def label(self, label: str) -> dict[str, object] | None:
        """The repository's label `label`, or None when it does not exist (HTTP 404)."""
        name = urllib.parse.quote(checked(LABEL, label, "label"), safe="")
        try:
            return as_dict(self.call("GET", f"labels/{name}"))
        except GhError as error:
            if NOT_FOUND in str(error):
                return None
            raise

    def create_label(self, label: str, color: str, description: str) -> dict[str, object]:
        """Creates the repository label `label` (colour as six hex digits, a short description)."""
        if not COLOR.fullmatch(color) or not description or len(description) > 100:
            raise GhError(f"refused label color or description for {label!r}")
        body = {"name": checked(LABEL, label, "label"), "color": color, "description": description}
        return as_dict(self.call("POST", "labels", body=body))

    def remove_label(self, pr: int | str, label: str) -> object:
        name = urllib.parse.quote(checked(LABEL, label, "label"), safe="")
        return self.call("DELETE", f"issues/{number(pr)}/labels/{name}")

    # CI.
    def check_runs(self, sha: str) -> list[object]:
        return self.pages(f"commits/{checked(SHA, sha, 'sha')}/check-runs", key="check_runs")

    def workflow_runs(
        self,
        *,
        branch: str | None = None,
        head_sha: str | None = None,
        event: str | None = None,
        created: str | None = None,
    ) -> list[object]:
        query = []
        if branch is not None:
            query.append(("branch", branch_name(branch)))
        if head_sha is not None:
            query.append(("head_sha", checked(SHA, head_sha, "sha")))
        if event is not None:
            query.append(("event", checked(EVENT, event, "event")))
        if created is not None:
            query.append(("created", checked(CREATED, created, "created")))
        return self.pages("actions/runs", query, key="workflow_runs")

    def run_jobs(self, run_id: int | str) -> list[object]:
        """The jobs of workflow run `run_id` (its latest attempt), with their timestamps."""
        return self.pages(f"actions/runs/{checked(RUN_ID, str(run_id), 'run id')}/jobs", key="jobs")

    def activity(self, branch: str, since: str) -> list[object]:
        """The pushes to `branch` (force pushes included), newest first, back past `since` (an
        ISO-8601 UTC instant). The activity endpoint ignores `page=`, so this one read follows its
        cursor, re-sent on the `repos/{owner}/{repo}` URL (module docstring)."""
        query = (("ref", f"refs/heads/{branch_name(branch)}"), ("per_page", str(PER_PAGE)))
        items: list[object] = []
        cursor: str | None = None
        for _ in range(MAX_PAGES):
            paged = (*query, ("after", cursor)) if cursor else query
            reply, cursor = self.send_cursor(Request("GET", f"{self.base}/activity", paged))
            if not isinstance(reply, list):
                raise GhError("expected a list from activity")
            items.extend(reply)
            last = reply[-1] if reply else None
            if cursor is None or not isinstance(last, dict) or str(last.get("timestamp", "")) < since:
                return items
        raise GhError(f"activity has more than {MAX_PAGES * PER_PAGE} items")

    def dispatch(self, workflow: str, ref: str, inputs: Mapping[str, str] | None = None) -> object:
        body: dict[str, object] = {"ref": branch_name(ref)}
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
        return self.parse(request, self.run(request, self.argv(request)))

    def send_cursor(self, request: Request) -> tuple[object, str | None]:
        argv = self.argv(request)
        out = self.run(request, [*argv[:2], "--include", *argv[2:]])
        head, _, body = out.replace("\r\n", "\n").partition("\n\n")
        found = NEXT_AFTER.search(head)
        cursor = urllib.parse.unquote(found[1]) if found else None
        if cursor is not None and not CURSOR.fullmatch(cursor):
            raise GhError(f"gh api {request.method} {request.path}: refused cursor")
        return self.parse(request, body), cursor

    def run(self, request: Request, argv: list[str]) -> str:
        """`gh api`'s output for `argv`; GhError when it fails."""
        stdin = json.dumps(request.body) if request.body is not None else ""
        try:
            result = subprocess.run(
                argv,
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
        return result.stdout

    @staticmethod
    def parse(request: Request, out: str) -> object:
        try:
            return json.loads(out) if out.strip() else None
        except ValueError as error:
            raise GhError(f"gh api {request.method} {request.path}: not JSON") from error


@dataclass
class Reply:
    """A canned answer of `FakeGh`: a JSON value, or the error to raise."""

    value: object = None
    error: str | None = None
    # The next page's cursor, for `send_cursor`.
    after: str | None = None


class FakeGh(Client):
    """Answers requests from `replies`, keyed by (method, url); records every request it gets."""

    def __init__(
        self, owner: str = "o", repo: str = "r", replies: Mapping[tuple[str, str], Reply] | None = None
    ) -> None:
        super().__init__(owner, repo)
        self.replies = dict(replies or {})
        self.requests: list[Request] = []

    def send(self, request: Request) -> object:
        return self.send_cursor(request)[0]

    def send_cursor(self, request: Request) -> tuple[object, str | None]:
        self.requests.append(request)
        reply = self.replies.get((request.method, request.url()))
        if reply is None:
            raise GhError(f"fake: no reply for {request.method} {request.url()}")
        if reply.error is not None:
            raise GhError(reply.error)
        return reply.value, reply.after


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
