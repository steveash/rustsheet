# Decision records

Short ADRs: context → decision → consequences. Required (enforced by `scripts/check_governance.py`)
for any change to protected paths (CI, `.claude/`, `CLAUDE.md`, lint policy, `scripts/`, `quality/`,
`docs/process/`) and for any baseline regression allowance. Create records with
`make new-decision TITLE="..."`: it writes `NNNN-slug.md` with the next number not used on the
working tree or any pushed `origin/*` branch (decision 0029). Push the record with the first commit
on your branch so other sessions see the number.
