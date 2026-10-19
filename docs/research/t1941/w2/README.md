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
