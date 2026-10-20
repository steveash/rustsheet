# W2 K2 x87 tools (T-1941)

Build: `gcc -O2 -shared -fPIC -o libx87.so x87.c -lm; gcc -O2 -shared -fPIC -o libs4r.so s4r.c -lm`.

Inputs:
- a `.npz` with `X` and `Y` (float64);
- a pickle `(p, q)`, coefficients highest degree first.

Any `(len p, len q)` works, so the tools also apply to K1 and K3. The kernel emulated is K2's `y = fl(t*fl(N/D))` with `t = x-2`.

## Tools
- **`x87.c` / `x87lib.py`** — kernel emulator, raw-x Horner. Every op runs in 80-bit. An op is stored to double unless
  it is named in `KEEP`.
  - Op names: `N`|`D` + `mul`|`add` + stage, plus `div`. Stage `i` multiplies by x, then adds `c[i]`.
  - `xl1` is a capped-L1 surrogate scorer.
- **`oprep.py`** — per-op repair test: leave one op unstored, count misses repaired minus hits broken.
  - It is a **localiser**. Within the stored-chain family the top op is the true one (a truth with only `Nadd3`
    unstored flags `Nadd3`).
  - But any early-rounding mismatch lights up the highest-weight early ops first, and so do coefficient errors in that
    part of the chain (W6's reverse control). On K2 these are `Nadd4` and `Dadd3`.
- **`s4read.py` / `s4r.c`** — per-point stage reader.
  - Takes the late chain as exact from stage `kN` / `kD` on.
  - Tests shifts of the stored `s_kN`, `d_kD` by `±A` / `±B` double ulps.
  - Outputs the admissible table `O[k, a, b]`.
- **`winfit_x.py`** — window fit under a `KEEP` structure with the late part frozen (`PE`/`QE` set the last free
  index). Freezing matters: unfrozen searches drift the late bits and stall even on true-structure controls.
- **`search.py`** — `nextd` and the within-polynomial null-direction segment search.

## K2 fingerprint table
Setup:
- Window: 2.40, 6 clusters x 512 consecutive doubles.
- Synthetic truth: truthX under the stated structure.
- Procedure: base-structure fit from a far start, then `oprep`.

| truth structure | base-fit ceiling | oprep top ops (net) |
|---|---|---|
| stored (base) | 97.7% | none (<= +1) |
| keep Nadd2 / Nmul3 | 97.8 / 97.2% | none |
| keep Nadd3 | 96.9% | Nadd3 +27 |
| keep Dadd2 | 97.5% | Dadd2 +6 |
| keep Nadd4 | 88.0% | Nadd4 +147 |
| keep Nadd4,Dadd3 | 86.4% | Nadd4 +154, Dadd3 +61, Nmul4 +7 |
| REAL (2.40) | 85.1% | Nadd4 +41 (only positive op) |
| REAL (50 x 0.02 windows) | 74-85% | Nadd4 top in 49/50, then Dadd3 / Nmul4 |

## Structure refits
Late part frozen; real data vs the structure's own far-start control:

| early structure | real | control |
|---|---|---|
| base (stored raw-x Horner) | 85.1% | 97.7% |
| Nadd4 + Dadd3 in 80-bit | 85.2% | 97.4% |
| all early ops in 80-bit | 85.5% | 98.6% |
| t = x-2 | 79.9% | 97.5% |
| t = x-2.5 | 79.8% | 96.9% |
| Estrin | 78.8% | 97.1% |
| FMA | 80.3% | 97.6% |
| power-sum | 79.8% | 96.2% |
| reverse power-sum | 73.9% | 97.5% |

## Caveat (retraction)
The late "pins" from the plateau reader are single-coordinate scans.
- `f = (x-2)N/D` has about 15 of 19 function-free directions at the ulp level. Most of them mix N and D and load on the
  late coefficients.
- So "late part = grid_a" holds only modulo those directions.
- The window solvers do NOT recover late or cross-N/D displacements, even on controls: truthL 38-64%, truthN 34-67%.

## Hand-off fingerprint tools (added later)
- **`dgrid.c` / `dgrid.py`** — per-cluster exhaustive scan of a smooth hand-off offset (dN, dD in ulps of `s_kN`,
  `d_kD`), with the late stored Horner exact. The overall best is the structure fingerprint at `(kN, kD)`.
  - Calibration: 5 constant sets for the same structure agree to ±0.006.
  - Build: `gcc -O2 -shared -fPIC -o libdgrid.so dgrid.c -lm`.
- **`lgreedy2.py`** — late-coefficient needle reader: greedy over each coefficient's full residue range, with dgrid as the
  inner scan. Control: from 4 simultaneous late errors it recovers to the base-structure ceiling of 94.3%.
- **`combo.c`** — synthetic truths with a per-chain early scheme (Horner, Estrin, FMA, power sums, 80-bit variants,
  even/odd, pairwise; see the header comment), followed by a stored raw-x Horner.

Fingerprints at (4,3) / (5,3) / (5,4), 2.40 window:
- REAL (late part = grid_a with p5 -4, q4 -1): 0.859 / 0.870 / 0.826 (2.80: 0.811 / 0.831 / 0.806).
- Base structure: 0.933-0.943 / 0.879-0.890 / 0.814-0.824.
- About 45 N-scheme x D-scheme combinations screened.
  - Closest: early even/odd in x^2 with the final add unstored. Still about 3 points too smooth at 2.80.
  - Whole-chain even/odd, product of quadratics, split N = A*x^5 + B, monic D and t-centred Horner are all rejected.
  - Constants scaled by powers of two (x/2, x/4) and an accumulator started at 0 give identical roundings to the base
    structure.

## Correction (late-set hygiene, HANDOFF 12.3 step 4)
Every REAL fingerprint row labelled "after greedy" (0.859 / 0.870 / 0.826) used grid_a with p5 -4 and q4 -1 applied as
coefficient ulps (nextafter). That is the window-local tweak, retracted globally because dense-run exact drops to 0.37.
The function-correct REAL rows use the committed, unmodified grid_a:
- 2.40 window: 0.818 / 0.856 / 0.826;
- 2.80 window: 0.842 / 0.860 / 0.806.

## Offset meter (HANDOFF 12.3 step 1)
Tools: `offm.c` / `offm.py`, in W2's scratchpad. Per run, a constant offset is added to the pre-rounding hand-off sums
(2-D grid, +-2.5 ulps, 1/32 step), with the late chain = grid_a.
- It recovers every function-preserving EARLY-only displacement to 0.92-0.96 on every run.
- It recovers LATE + cross-N/D displacements only partially and erratically.
- REAL, 14 long runs: 0.66-0.78 -> 0.71-0.83, never near 0.92.

So the late chain itself differs from grid_a by a joint cross-N/D move, (a) in HANDOFF 12.3. Single-coordinate
plateau, needle and line readers cannot see such a move.
