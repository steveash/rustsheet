# T-3571 r5-bfrac: Excel's bfrac fraction arithmetic

(Draft: the results are filled in below once the model is frozen.)

## Hold-out plan (fixed before any hold-out scoring, 2026-11-02)
- Split by shape, built and filled together (mkprobe.py): t3571-r5-bfrac-tune (6 shapes, 1,536 cells) is the only
  probe any choice is scored on; t3571-r5-bfrac-hold (6 other shapes, 1,632 cells) is scored once, after model.py is
  frozen and committed.
- Margin: the hold-out bit-exact rate must be within 10 percentage points of the tune rate, and above the correctly
  rounded value's hold-out bit-exact count.
- Fresh sweep: after the freeze, a new probes-numeric case of log-uniform (a, b) in the bfrac region with random x,
  oracle-filled, scored once, with its Gate A envelope count against 50-digit math.
