# W2 tools (K2 and K3), committed at the 2026-10-20 pause

W2 pasted these from its scratchpad; they assume the repo root as cwd and `review3/spec.py` for `load`.
- `k2ref.py`: K2 reference model and end-to-end check (clusters plus 14 long runs). For the R3 basin, use the
  hex in `../k2_ref.py`. W2's alternative basin scores 0.9522.
- `nullfit5.py`: LLL near-null greedy climb (env START=pkl of (P,Q), UN stage modes, IDXN/IDXD, RND, PLANT, TAG).
- `hvar.py` + `entry.py`: c/x entry-point scan (forms E1..E5, Eb, Ec, J0) with planted controls.
- `opplain.py`: per-stage op-mode greedy (S / U / Pst / Ast).
- `hoff.c` + `gwide.py`: wide 1-D head-coefficient scan with an exact LS re-solve of the lower coefficients.
  Build: `gcc -O2 -mfpmath=387 -shared -fPIC -o libhoff.so hoff.c`.
