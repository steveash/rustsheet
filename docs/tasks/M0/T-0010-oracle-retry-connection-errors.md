---
id: T-0010
title: "Oracle client: retry transient connection errors"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
A transient network failure during `rustsheet-oracle fill` is retried instead of aborting the run.

## Context
During T-0009 a `fill` crashed with `requests.exceptions.ConnectionError` (connection reset by
peer). `GraphBackend._call` in `tools/src/rustsheet_tools/oracle/backends.py` retries only on HTTP
429/503/504; connection errors escape uncaught and are not turned into `OracleError`, so `fill`
stops without writing its summary. A re-run succeeded.

## Acceptance criteria
- [ ] `_call` retries `requests.ConnectionError`/`Timeout` with the existing backoff, then raises `OracleError` (redacted) when retries are exhausted.
- [ ] Unit test with a fake session covering retry-then-success and exhaustion.
- [ ] `make check` green; reviewers approve.

## Out of scope
Changing the retry policy for HTTP status codes.
