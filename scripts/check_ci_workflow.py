#!/usr/bin/env python3
"""Fails when `.github/workflows/ci.yml` holds more rights than PR CI needs (decision 0147).

CI runs untrusted code: every PR's, and since SPEC-0027 section 9 every merge train's pushed to
`merge-train/current` or `merge-train/shadow`. So `ci.yml` must not read a secret or the job
token (`secrets.`, `secrets[...]`, `secrets:`, `github.token`), and every `permissions` block,
including the required workflow-level one, must be exactly `contents: read`.

`push_branches` also reads the workflow's `on.push.branches`, which `test_check_ci_workflow.py`
pins. Standard library only (the hygiene job has no PyYAML), so it parses only the block-style
subset of YAML that `ci.yml` uses; anything it cannot read as `contents: read` fails.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

CI_YML = Path(__file__).resolve().parent.parent / ".github" / "workflows" / "ci.yml"

ALLOWED_PERMISSIONS = ["contents: read"]

# A reference to a secret or the job token, in an expression, a `with:`/`env:` value or a
# reusable-workflow call (`secrets: inherit`). Comments count too: the check errs towards failing.
SECRET_REFERENCES = (
    re.compile(r"\bsecrets\s*[.\[]"),
    re.compile(r"^\s*secrets\s*:"),
    re.compile(r"\$\{\{[^}]*\bsecrets\b"),
    re.compile(r"\bgithub\s*(\.\s*token\b|\[\s*['\"]token)"),
)

PERMISSIONS = re.compile(r"^(?P<indent> *)permissions\s*:(?P<value>.*)$")


def _content(line: str) -> str:
    """`line` without its trailing comment and whitespace (no `#` appears inside these values)."""
    return line.split("#", 1)[0].rstrip()


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _block(lines: list[str], start: int, indent: int) -> list[tuple[int, str]]:
    """The non-blank, non-comment lines after `start` indented deeper than `indent`."""
    out = []
    for i in range(start + 1, len(lines)):
        text = _content(lines[i])
        if not text.strip():
            continue
        if _indent(text) <= indent:
            break
        out.append((i, text.strip()))
    return out


def violations(text: str) -> list[str]:
    """One message per line that references a secret or widens a `permissions` block."""
    lines = text.splitlines()
    found = []
    top_level = False
    for n, line in enumerate(lines, 1):
        if any(p.search(line) for p in SECRET_REFERENCES):
            found.append(f"ci.yml:{n}: references a secret or the job token")
        m = PERMISSIONS.match(_content(line))
        if m is None:
            continue
        indent = len(m["indent"])
        top_level |= indent == 0
        entries = [entry for _, entry in _block(lines, n - 1, indent)]
        if m["value"].strip() or entries != ALLOWED_PERMISSIONS:
            found.append(f"ci.yml:{n}: permissions must be exactly `contents: read`")
    if not top_level:
        found.append("ci.yml: no workflow-level `permissions: contents: read` block")
    return found


def _unquote(item: str) -> str:
    return item.strip().strip("'\"")


def push_branches(text: str) -> list[str] | None:
    """`on.push.branches` as written (flow or block list), or None when it is not found."""
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if _content(line) not in ("on:", '"on":', "'on':"):
            continue
        for j, entry in _block(lines, i, 0):
            if entry != "push:":
                continue
            for k, sub in _block(lines, j, _indent(lines[j])):
                key, _, value = sub.partition(":")
                if key.strip() != "branches":
                    continue
                value = value.strip()
                if value.startswith("[") and value.endswith("]"):
                    return [_unquote(b) for b in value[1:-1].split(",") if b.strip()]
                items = _block(lines, k, _indent(lines[k]))
                return [_unquote(item[1:]) for _, item in items if item.startswith("-")]
    return None


def main() -> int:
    found = violations(CI_YML.read_text(encoding="utf-8"))
    for message in found:
        print(message, file=sys.stderr)
    if found:
        print("CI must run with `contents: read` and no secrets (decision 0147)", file=sys.stderr)
        return 1
    print("ci workflow rights ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
