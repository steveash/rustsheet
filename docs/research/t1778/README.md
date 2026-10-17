# T-1778 erf kernel research scripts

Data: `python3 extract_runs.py OUT` writes `OUT/runs.txt` and `OUT/zs3.txt` from the probe sidecars.
Build `covm.c` and `covd.c` into OUT (`cc -O2 covm.c -lm -o covm`, `cc -O2 covd.c -lquadmath -lm -o covd`) and run
the Python scripts from OUT (scipy for `hboot.py`, mpmath for `wfamilies.py`).

- `hboot.py`: bootstraps the coarse polynomial H(z) from the z where hi = fl(H) steps (statistics.md, T-1778).
- `wsweep.py`, `wsweep_small.py`, `wphase.py`: read the W rounding sawtooth's phase at each sweep.
- `wfamilies.py`: tests one-parameter forms of W against those phases; only sqrt(1 - 2bz) survives.
- `erf_small_model.c`, `erf_global_fit.c`: the earlier small-z model and global fit.
- `sweepphase.py`, `phscan.c`: per-sweep W phases by exact interval intersection; `w_phases.txt` holds the
  28 sweep phases (name, z of the sweep's first run, phase interval) that `wfit.py` chains into W's polynomial.
