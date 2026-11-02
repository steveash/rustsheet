/* T-3571: search over bfrac fraction loops (TOMS forward recurrence and Lentz forms) on TOMS 708's terms, SSE doubles.
   stdin as search.c ("N" then "a b x y P cdf rxhi rxlo ee"). Prints one line per (variant, assembly):
   variant-code assembly exact corr mean sd.
   Term choices: W 3 (n*(b-n)*x, n*x*(b-n), n*((b-n)*x)), AL 4, E2 2, BE 3.
   Loop choices LP: 0..5 TOMS (update {plain, fma(al,an,be*anp1), fma(be,anp1,al*an)} x rescale {divide, 1/bnp1 shared}),
   6.. Lentz: D-update {be+al*D, fma(al,D,be)} x C-update {be+al/C, fma? no: be+al*(1/C)} x delta {C*D after D=1/D,
   C/D without inverting}; Lentz output F = c/c1 + K(al/be); r = 1/F.
   EPS 3: 1.5e-14, 1e-15, 2.2e-16.
   Assembly AS: 0 fl(fl(P*fl(x*y))*r); 1 fl(fl(P*fl(x*y))/F) (F = 1/r for TOMS: bnp1/anp1 not formed; uses fl(1/r));
   2 fl(fl(fl(P*x)*y)*r); 3 fl(P*fl(fl(x*y)*r)). */
#include <math.h>
#include <stdio.h>
#define NMAX 4096
static double A[NMAX], B[NMAX], X[NMAX], Y[NMAX], P[NMAX], CDF[NMAX], RH[NMAX], RL[NMAX], EE[NMAX];
enum { NW = 3, NAL = 4, NE2 = 2, NBE = 3, NLP = 6 + 8, NEPS = 3, NAS = 4 };
static void frac(double a, double b, double x, double y, int w_, int al_, int e2_, int be_, int lp, double eps,
                 double *rout, double *Fout) {
  double lambda = a - (a + b) * x;
  double c = lambda + 1., c0 = b / a, c1 = 1. / a + 1., yp1 = y + 1., n = 0., p = 1., s = a + 1.;
  double an = 0., bn = 1., anp1 = 1., bnp1 = c / c1, r = c1 / c, r0;
  double f = c / c1, C = f, D = 0.;
  for (int k = 0; k < 2000; k++) {
    double w, t, e, alpha, beta, X2, ws;
    n += 1.;
    switch (w_) { case 0: w = n * (b - n) * x; break; case 1: w = n * x * (b - n); break; default: w = n * ((b - n) * x); }
    t = n / a; e = a / s;
    switch (al_) {
      case 0: alpha = p * (p + c0) * e * e * (w * x); break;
      case 1: alpha = (p * (p + c0)) * (e * e) * (w * x); break;
      case 2: alpha = p * (p + c0) * e * e * w * x; break;
      default: alpha = (p * (p + c0)) * ((e * e) * (w * x)); }
    e = e2_ ? (t + 1.) / (c1 + 2. * t) : (t + 1.) / (c1 + t + t);
    X2 = c + n * yp1; ws = w / s;
    switch (be_) { case 0: beta = n + ws + e * X2; break; case 1: beta = ws + (n + e * X2); break; default: beta = n + (ws + e * X2); }
    p = t + 1.; s += 2.;
    if (lp < 6) {
      int up = lp % 3, rc = lp / 3;
      if (up == 0) { t = alpha * an + beta * anp1; an = anp1; anp1 = t; t = alpha * bn + beta * bnp1; bn = bnp1; bnp1 = t; }
      else if (up == 1) { t = fma(alpha, an, beta * anp1); an = anp1; anp1 = t; t = fma(alpha, bn, beta * bnp1); bn = bnp1; bnp1 = t; }
      else { t = fma(beta, anp1, alpha * an); an = anp1; anp1 = t; t = fma(beta, bnp1, alpha * bn); bn = bnp1; bnp1 = t; }
      r0 = r;
      double inv = 1. / bnp1;
      r = rc ? anp1 * inv : anp1 / bnp1;
      if (fabs(r - r0) <= eps * r) break;
      if (rc) { an *= inv; bn *= inv; } else { an /= bnp1; bn /= bnp1; }
      anp1 = r; bnp1 = 1.;
    } else {
      int q = lp - 6, du = q & 1, cu = (q >> 1) & 1, dl = (q >> 2) & 1, del;
      double delta;
      if (!dl) {
        D = du ? fma(alpha, D, beta) : beta + alpha * D; if (D == 0) D = 1e-300;
        C = cu ? beta + alpha * (1. / C) : beta + alpha / C; if (C == 0) C = 1e-300;
        D = 1. / D; delta = C * D;
      } else {  /* keep Dn = B_n/B_{n-1} un-inverted: delta = C/D */
        D = du ? fma(alpha, k ? 1. / D : 0., beta) : beta + (k ? alpha / D : 0.);
        C = cu ? beta + alpha * (1. / C) : beta + alpha / C;
        delta = C / D;
      }
      f *= delta; (void)del;
      if (fabs(delta - 1.) <= eps) break;
    }
  }
  if (lp < 6) { *rout = r; *Fout = 1. / r; } else { *rout = 1. / f; *Fout = f; }
}
int main(void) {
  int N; if (scanf("%d", &N) != 1 || N > NMAX) return 1;
  for (int i = 0; i < N; i++)
    if (scanf("%la %la %la %la %la %la %la %la %la", &A[i], &B[i], &X[i], &Y[i], &P[i], &CDF[i], &RH[i], &RL[i], &EE[i]) != 9) return 1;
  const double epss[NEPS] = {1.5e-14, 1e-15, 2.220446049250313e-16};
  static double rr[NMAX], FF[NMAX];
  for (int w_ = 0; w_ < NW; w_++) for (int al_ = 0; al_ < NAL; al_++) for (int e2_ = 0; e2_ < NE2; e2_++)
  for (int be_ = 0; be_ < NBE; be_++) for (int lp = 0; lp < NLP; lp++) for (int ei = 0; ei < NEPS; ei++) {
    double sm = 0, sm2 = 0, se = 0, se2 = 0, sxy = 0;
    for (int i = 0; i < N; i++) {
      frac(A[i], B[i], X[i], Y[i], w_, al_, e2_, be_, lp, epss[ei], &rr[i], &FF[i]);
      double err = ((rr[i] - RH[i]) - RL[i]) / RH[i] * 9007199254740992.0;
      sm += err; sm2 += err * err; se += EE[i]; se2 += EE[i] * EE[i]; sxy += err * EE[i];
    }
    double mm = sm / N, sd = sqrt(sm2 / N - mm * mm), me = se / N, sde = sqrt(se2 / N - me * me);
    double corr = (sxy / N - mm * me) / (sd * sde);
    for (int as = 0; as < NAS; as++) {
      int ex = 0;
      for (int i = 0; i < N; i++) {
        double xy = X[i] * Y[i], v;
        switch (as) { case 0: v = (P[i] * xy) * rr[i]; break; case 1: v = (P[i] * xy) / FF[i]; break;
                      case 2: v = ((P[i] * X[i]) * Y[i]) * rr[i]; break; default: v = P[i] * (xy * rr[i]); }
        ex += v == CDF[i];
      }
      printf("W%d AL%d E%d BE%d LP%d EPS%d AS%d %d %.4f %.3f %.3f\n", w_, al_, e2_, be_, lp, ei, as, ex, corr, mm, sd);
    }
  }
  return 0;
}
