# T-1941 W2 tools: GAMMALN base kernel on (2,3]

- poly.c and ysearch.py: exact scorer and function-preserving segment search.
- anch.py: joint multi-anchor early-stage search.
- off.c: stage-offset probe.
- Not committed (descriptions in W2's reports, logged in ../README.md):
  - multinull2.py, the 137-cluster version of anch.py;
  - msk.c, the unrounded-op and final-op variants;
  - pw.c, the pointwise solvability bitmap and residual spectrum.
- Best global set: grid_a.txt.

Conclusions (2026-10-19):
- **The late stages are pinned.** The last 4 of each chain need 0 offset at every anchor.
- **The early stages fit one anchor at a time, never jointly.** Any single anchor reaches 98.6-99.97% exact
  through near-null early-stage moves. Jointly, 10 anchors average about 81-82% and no set has more than 2-3
  anchors near 95%.
- **There is no breakpoint.** In 0.01 bins the per-anchor sets swap rank with no step.
- **So the raw-x (9,8) Horner form is very likely not Excel's form.** (8,8) does worse (32-42%).
- **Untested:** an Estrin or split early part, a different early-stage variable (x-2, x^2), or other degrees.

## W2 tools (K2 and K3), committed at the 2026-10-20 pause

W2 pasted these from its scratchpad; they assume the repo root as cwd and `review3/spec.py` for `load`.
- `k2ref.py`: K2 reference model and end-to-end check (clusters plus 14 long runs). For the R3 basin, use the
  hex in `../k2_ref.py`. W2's alternative basin scores 0.9522.
- `nullfit5.py`: LLL near-null greedy climb (env START=pkl of (P,Q), UN stage modes, IDXN/IDXD, RND, PLANT, TAG).
- `hvar.py` + `entry.py`: c/x entry-point scan (forms E1..E5, Eb, Ec, J0) with planted controls.
- `opplain.py`: per-stage op-mode greedy (S / U / Pst / Ast).
- `hoff.c` + `gwide.py`: wide 1-D head-coefficient scan with an exact LS re-solve of the lower coefficients.
  Build: `gcc -O2 -mfpmath=387 -shared -fPIC -o libhoff.so hoff.c`.
