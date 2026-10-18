> Copied by T-1916 from branch agent/research/T-1778-kernel-map at 59c2c3724a3c (path docs/research/t1778/round3/README.md), unchanged below this line.

# T-1778 round 3

- `sol100.txt`: the bit-exact small-x erf coefficients (P quartic, Q quintic, hex doubles). Score them with
  `../round2/sc.c` in mode 0300: 775,272 of 775,272 oracle points for x <= 0.5.
- `erf-search/`: the interval-inversion and LP scripts that found them (research code; paths inside point at
  the session scratch directory and the run files written by `../extract_runs.py`).
- `erfc/`: the ERFC-tail work.
  - `PROBES.md` documents every `kmyst-erfc-*` probe case, and `gen_*.py` generate them.
  - `expx87*.py` hold the x87 EXP emulation.
  - `reach*.py` and `infer_r.py` test ERFC = fl87(E*R) and recover the stored double R.
