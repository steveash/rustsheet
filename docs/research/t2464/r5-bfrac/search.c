/* T-3571: exhaustive op-order search over TOMS 708 bfrac's fraction in SSE doubles.
   stdin: header "N", then N lines "a b x y P cdf rxhi rxlo ee" (%la; ee = Excel implied fraction error, units 2^-53).
   For each variant v prints: v exact corr mean sd (exact: fl(fl(P*fl(x*y))*r) == cdf; corr with ee; model error vs rx).
   Choice points (mixed radix): W 3, AL 4, E2 2, BE 3, UP 3, FMA-in-beta 2, recip rescale+r 2, EPS 2, t=n*(1/a) 2, c0=b*(1/a) 2, 1/s shared 2.  Build: cc -O2 -mfma -ffp-contract=off */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#define NMAX 4096
static double A[NMAX], B[NMAX], X[NMAX], Y[NMAX], P[NMAX], CDF[NMAX], RH[NMAX], RL[NMAX], EE[NMAX];
static int CH[11] = {3, 4, 2, 3, 3, 2, 2, 2, 2, 2, 2};
static double frac(double a, double b, double x, double y, const int *o) {
  double lambda = a - (a + b) * x;
  double c = lambda + 1., c0 = b / a, c1 = 1. / a + 1., yp1 = y + 1., n = 0., p = 1., s = a + 1., an = 0., bn = 1.,
         anp1 = 1., bnp1 = c / c1, r = c1 / c, r0, w, t, e, alpha, beta, X2, ee;
  double eps = o[7] ? 2.220446049250313e-16 : 1.5e-14, ia = 1. / a, is;
  if (o[9]) c0 = b * ia;
  for (int k = 0; k < 1000; k++) {
    n += 1.;
    switch (o[0]) { case 0: w = n * (b - n) * x; break; case 1: w = n * x * (b - n); break; default: w = n * ((b - n) * x); }
    t = o[8] ? n * ia : n / a; is = 1. / s; e = o[10] ? a * is : a / s;
    switch (o[1]) {
      case 0: alpha = p * (p + c0) * e * e * (w * x); break;
      case 1: alpha = (p * (p + c0)) * (e * e) * (w * x); break;
      case 2: alpha = p * (p + c0) * e * e * w * x; break;
      default: alpha = (p * (p + c0)) * ((e * e) * (w * x)); }
    e = o[2] ? (t + 1.) / (c1 + 2. * t) : (t + 1.) / (c1 + t + t);
    X2 = (o[2] & 1) && 0 ? 0 : c + n * yp1;
    double ws = o[10] ? w * is : w / s;
    if (o[5] == 0) {
      switch (o[3]) { case 0: beta = n + ws + e * X2; break; case 1: beta = ws + (n + e * X2); break; default: beta = n + (ws + e * X2); }
    } else {
      X2 = fma(n, yp1, c);
      switch (o[3]) { case 0: beta = fma(e, X2, n + ws); break; case 1: beta = ws + fma(e, X2, n); break; default: beta = n + fma(e, X2, ws); }
    }
    p = t + 1.; s += 2.;
    if (o[4] == 0) { t = alpha * an + beta * anp1; an = anp1; anp1 = t; t = alpha * bn + beta * bnp1; bn = bnp1; bnp1 = t; }
    else if (o[4] == 1) { t = fma(alpha, an, beta * anp1); an = anp1; anp1 = t; t = fma(alpha, bn, beta * bnp1); bn = bnp1; bnp1 = t; }
    else { t = fma(beta, anp1, alpha * an); an = anp1; anp1 = t; t = fma(beta, bnp1, alpha * bn); bn = bnp1; bnp1 = t; }
    r0 = r;
    r = o[6] ? anp1 * (1. / bnp1) : anp1 / bnp1;
    if (fabs(r - r0) <= eps * r) break;
    if (o[6] == 0) { an /= bnp1; bn /= bnp1; } else { ee = 1. / bnp1; an *= ee; bn *= ee; }
    anp1 = r; bnp1 = 1.;
  }
  return r;
}
int main(void) {
  int N; if (scanf("%d", &N) != 1 || N > NMAX) return 1;
  for (int i = 0; i < N; i++)
    if (scanf("%la %la %la %la %la %la %la %la %la", &A[i], &B[i], &X[i], &Y[i], &P[i], &CDF[i], &RH[i], &RL[i], &EE[i]) != 9) return 1;
  int tot = 1; for (int k = 0; k < 11; k++) tot *= CH[k];
  for (int v = 0; v < tot; v++) {
    int o[11], q = v; for (int k = 0; k < 11; k++) { o[k] = q % CH[k]; q /= CH[k]; }
    int ex = 0; double sm = 0, sm2 = 0, se = 0, se2 = 0, sxy = 0;
    for (int i = 0; i < N; i++) {
      double r = frac(A[i], B[i], X[i], Y[i], o);
      ex += (P[i] * (X[i] * Y[i])) * r == CDF[i];
      double err = ((r - RH[i]) - RL[i]) / RH[i] * 9007199254740992.0;
      sm += err; sm2 += err * err; se += EE[i]; se2 += EE[i] * EE[i]; sxy += err * EE[i];
    }
    double mm = sm / N, sd = sqrt(sm2 / N - mm * mm), me = se / N, sde = sqrt(se2 / N - me * me);
    printf("%d %d %.4f %.3f %.3f ", v, ex, (sxy / N - mm * me) / (sd * sde), mm, sd); for (int k = 0; k < 11; k++) printf("%d", o[k]); printf("\n");
  }
  return 0;
}
