# Engine fixtures for the T-1941 kernels (T-2123, T-2128)

Scripts that build the engine's committed kernel fixtures (`crates/rustsheet-engine/src/test/fixtures/`) from the T-1941
oracle runs on branch `agent/research/T-1941-crack-gammaln-gamma-ln`. They need that branch fetched, Python 3 with numpy
(longdouble is the x87 80-bit format on x86-64) and mpmath, and `rnd.py`, `ss_fc.txt` and `xl_chain_d3ls.txt` from the
research directory copied next to them (`DIR` below).

1. `python3 extract.py DIR` streams every `kmyst-gam-*` and `t1042-gamma-*` case from the branch (`git cat-file`) and writes
   `DIR/{ln,gl,ga}.rows` (`x_hex y_hex case`, one line per cell whose formula is `=F(A<row>)`). `RUSTSHEET_T1941_ROWS=DIR`
   lets the ignored full-run tests read them. For GAMMALN, sort to distinct x first: `LC_ALL=C sort -u -k1,2 gl.rows > gl.u2`.
2. LN: `python3 mkfixture_ln.py DIR DIR/ln.rows ln_kernel.fixture full` (4,000 random distinct x plus every model miss).
3. GAMMALN: `python3 sample_gl.py DIR/gl.u2 DIR/gl.sample 1500`, then `python3 score_sample.py DIR x 0.01` writes
   `DIR/gl.fixture` with `glref.py`'s value per point. `glref.py` composes the research checkers (`k2_ref.py`, reviewer 3's
   cand9, `w5/p5/maskmodel.py` all-stored for K3, `w6/synth.py` for (1.5,2], `gammaln_big_check.c`, `lnmodels.py`) with
   the region ends Excel's cells show (docs/excel-semantics/statistics.md "GAMMALN is Excel's own x87 routine").
4. `python3 cand9.py DIR` measures the cand9 cut for K2 near 2 (the table in statistics.md).
