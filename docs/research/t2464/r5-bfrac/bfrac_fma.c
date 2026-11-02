/* T-3571: TOMS 708 bfrac's fraction r (I = brcomp * r) in SSE doubles, with FMA contraction variants.
   stdin: "a b x y" as %la; stdout per line: r for each (mask, eps) as %a, mask-major.
   mask bits: 1 c+n*yp1 fused; 2 (n+w/s)+e*X fused; 4 t=fma(alpha,an,beta*anp1); 8 t=fma(beta,anp1,alpha*an)
   (4 and 8 exclusive); 16 lambda = fma(-(a+b),x,a); 32 Fortran order (w = n*(b-n)*x, alpha with p*(p+c0)*e*e
   grouped); 64 r test as fma(-eps, r, |r - r0|) <= 0. Build: cc -O0 -mfma -ffp-contract=off. */
#include <math.h>
#include <stdio.h>
static double bfrac_r(double a, double b, double x, double y, double lambda, double eps, int m, int *it) {
  double c = 1.0 + lambda, c0 = b / a, c1 = 1.0 + 1.0 / a, yp1 = y + 1.0, n = 0, p = 1, s = a + 1, an = 0, bn = 1,
         anp1 = 1, bnp1 = c / c1, r = c1 / c, r0, t, w, e, alpha, beta, X;
  int k = 0;
  for (;;) {
    n += 1.0; k++;
    t = n / a;
    w = (m & 32) ? n * (b - n) * x : n * (b - n) * x;
    e = a / s;
    alpha = (m & 32) ? (p * (p + c0) * e * e) * (w * x) : p * (p + c0) * e * e * (w * x);
    e = (1.0 + t) / (c1 + t + t);
    X = (m & 1) ? fma(n, yp1, c) : c + n * yp1;
    beta = (m & 2) ? fma(e, X, n + w / s) : n + w / s + e * X;
    p = 1.0 + t;
    s += 2.0;
    if (m & 4) t = fma(alpha, an, beta * anp1); else if (m & 8) t = fma(beta, anp1, alpha * an); else t = alpha * an + beta * anp1;
    an = anp1; anp1 = t;
    if (m & 4) t = fma(alpha, bn, beta * bnp1); else if (m & 8) t = fma(beta, bnp1, alpha * bn); else t = alpha * bn + beta * bnp1;
    bn = bnp1; bnp1 = t;
    r0 = r; r = anp1 / bnp1;
    int stop = (m & 64) ? fma(-eps, r, fabs(r - r0)) <= 0 : fabs(r - r0) <= eps * r;
    if (stop || k >= 10000) break;
    an /= bnp1; bn /= bnp1; anp1 = r; bnp1 = 1.0;
  }
  *it = k;
  return r;
}
int main(void) {
  double a, b, x, y;
  const double epss[] = {1.5e-14, 1e-15, 2.220446049250313e-16};
  while (scanf("%la %la %la %la", &a, &b, &x, &y) == 4) {
    int it = 0;
    for (int m = 0; m < 128; m++) {
      if ((m & 12) == 12) continue;
      double lam = (m & 16) ? fma(-(a + b), x, a) : a - (a + b) * x;
      for (int ei = 0; ei < 3; ei++) printf("%a ", bfrac_r(a, b, x, y, lam, epss[ei], m, &it));
    }
    printf("%d\n", it);
  }
  return 0;
}
