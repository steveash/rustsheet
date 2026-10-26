// TOMS 708 BFRAC fraction r (I = brcomp*r) variants. Reads "a b x y" hex; prints r for each variant (%La).
// variants: ord (0 = R's C ordering, 1 = original Fortran ordering), lam (0: a-(a+b)x, 1: (a+b)y-b, 2: TOMS rule),
// eps list. MODE: 0 SSE double, 1 per-op stored x87 (Dx), 2 long double, 3 double built with -mfpmath=387 -ffloat-store
#include <initializer_list>
#include "cf.hpp"
#ifndef MODE
#define MODE 0
#endif
#if MODE == 1
typedef Dx T;
#elif MODE == 2
typedef long double T;
#else
typedef double T;
#endif
template <class TT> TT bfrac_r(TT a, TT b, TT x, TT y, TT lambda, double eps, int ord, int maxit, int *iters) {
  TT c = lambda + TT(1), c0 = b / a, c1 = TT(1) / a + TT(1), yp1 = y + TT(1), n = TT(0), p = TT(1), s = a + TT(1),
     an = TT(0), bn = TT(1), anp1 = TT(1), bnp1 = c / c1, r = c1 / c, r0;
  if (ord == 1) { c = TT(1) + lambda; c1 = TT(1) + TT(1) / a; bnp1 = c / c1; r = c1 / c; }
  int k = 0;
  for (;;) {
    n += TT(1); k++;
    TT t = n / a, w, e = a / s, alpha, beta;
    if (ord == 0) {
      w = n * x * (b - n);
      alpha = p * (p + c0) * e * e * (w * x);
      e = (t + TT(1)) / (c1 + t + t);
      beta = w / s + (n + e * (c + n * yp1));
      p = t + TT(1);
    } else {
      w = n * (b - n) * x;
      alpha = (p * (p + c0) * e * e) * (w * x);
      e = (TT(1) + t) / (c1 + t + t);
      beta = n + w / s + e * (c + n * yp1);
      p = TT(1) + t;
    }
    s += TT(2);
    t = alpha * an + beta * anp1; an = anp1; anp1 = t;
    t = alpha * bn + beta * bnp1; bn = bnp1; bnp1 = t;
    r0 = r; r = anp1 / bnp1;
    if (eps >= 0 ? absT(r - r0) <= TT(eps) * r : false) break;
    if (k >= maxit) break;
    an /= bnp1; bn /= bnp1; anp1 = r; bnp1 = TT(1);
  }
  if (iters) *iters = k;
  return r;
}
int main() {
  double a, b, x, y;
  while (scanf("%la %la %la %la", &a, &b, &x, &y) == 4) {
    T A(a), B(b), X(x), Y(y);
    int it;
    for (int ord : {0, 1})
      for (int lm : {0, 1, 2, 3}) {
        volatile double ab = a + b, abx = ab * x, ld = a - abx;
        T lam = lm == 0 ? A - (A + B) * X : lm == 1 ? (A + B) * Y - B : lm == 2 ? (a > b ? (A + B) * Y - B : A - (A + B) * X) : T(ld);
        for (double e : {1.5e-14, 1e-15, 2.220446049250313e-16, 1e-14, 3e-15})
          printf("%La ", LD(bfrac_r<T>(A, B, X, Y, lam, e, ord, 1000, &it)));
      }
    printf("%d\n", it);
  }
}
