// bfrac with mixed precision: long double arithmetic, variables in groups rounded to double on assignment per mask.
// G1 (1): c,c0,c1,yp1   G2 (2): w,t,e   G4 (4): alpha,beta   G8 (8): an,bn,anp1,bnp1   G16 (16): r   G32 (32): p,s
// ord 0 = R C order, 1 = Fortran order.  lambda passed as double fl(a - fl((a+b)x)).  eps fixed per run (argv).
#include <cstdio>
#include <cstdlib>
#include <cmath>
typedef long double L;
static inline L rd(L v, int on) { if (!on) return v; volatile double d = (double)v; return d; }
L bf(double a_, double b_, double x_, double y_, double lam_, double eps, int ord, unsigned m, int *it) {
  L a = a_, b = b_, x = x_, y = y_, lambda = lam_;
  int g1 = m & 1, g2 = m & 2, g4 = m & 4, g8 = m & 8, g16 = m & 16, g32 = m & 32;
  L c = rd(lambda + 1, g1), c0 = rd(b / a, g1), c1 = rd(ord ? 1 + 1 / a : 1 / a + 1, g1), yp1 = rd(y + 1, g1);
  L n = 0, p = 1, s = rd(a + 1, g32), an = 0, bn = 1, anp1 = 1, bnp1 = rd(c / c1, g8), r = rd(c1 / c, g16), r0;
  int k = 0;
  for (;;) {
    n += 1; k++;
    L t = rd(n / a, g2), w, e = rd(a / s, g2), alpha, beta;
    if (ord == 0) {
      w = rd(n * x * (b - n), g2);
      alpha = rd(p * (p + c0) * e * e * (w * x), g4);
      e = rd((t + 1) / (c1 + t + t), g2);
      beta = rd(w / s + (n + e * (c + n * yp1)), g4);
      p = rd(t + 1, g32);
    } else {
      w = rd(n * (b - n) * x, g2);
      alpha = rd((p * (p + c0) * e * e) * (w * x), g4);
      e = rd((1 + t) / (c1 + t + t), g2);
      beta = rd(n + w / s + e * (c + n * yp1), g4);
      p = rd(1 + t, g32);
    }
    s = rd(s + 2, g32);
    t = rd(alpha * an + beta * anp1, g8); an = anp1; anp1 = t;
    t = rd(alpha * bn + beta * bnp1, g8); bn = bnp1; bnp1 = t;
    r0 = r; r = rd(anp1 / bnp1, g16);
    if (std::fabs(r - r0) <= eps * r) break;
    if (k >= 1000) break;
    an = rd(an / bnp1, g8); bn = rd(bn / bnp1, g8); anp1 = r; bnp1 = 1;
  }
  *it = k;
  return r;
}
int main(int argc, char **argv) {
  double eps = atof(argv[1]);
  double a, b, x, y;
  while (scanf("%la %la %la %la", &a, &b, &x, &y) == 4) {
    volatile double ab = a + b, abx = ab * x, lam = a - abx;
    int it;
    for (int ord = 0; ord < 2; ord++)
      for (unsigned m = 0; m < 64; m++) printf("%La ", bf(a, b, x, y, lam, eps, ord, m, &it));
    printf("%d\n", it);
  }
}
