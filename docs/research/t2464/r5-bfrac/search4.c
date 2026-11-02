/* T-3571: Lentz-family search (NR-style starts ST 1-6: tiny 1e-30, 1e-300, DBL_MIN, 1e-100; f = 1/b0; f = tiny*C*D left-assoc) for bfrac's fraction F = c/c1 + K(alpha_n/beta_n) on TOMS 708's terms, SSE doubles.
   stdin as search.c. One line per variant: code exact(AS0) exact(AS1) corr mean sd.
   Terms: W 3, AL 2 (left-assoc; (p(p+c0))((e e)(w x))), E2 2 (c1+t+t, c1+2t), BE 3.
   C update CU 4: be+al/C; fma(al,1/C,be); be+al*(1/C); be + (al/C) with C kept as 1/C (Cinv=1/(be*Cinv... no) -> fma(al,Ci,be) Ci=1/C.
   D form DF 6: 0 D=1/(be+al*D) [modified Lentz]; 1 D=1/fma(al,D,be); 2 D=be+al/D; 3 D=fma(al,1/D,be); 4 D=be+al*(1/D);
   5 D=fma(al, Di, be), Di=1/D kept.
   Delta DL 3: C*Dinv (Dinv=D for DF 0,1 else 1/D); C/D (DF>=2; for DF0/1 C/(1/D) not formed -> C*D); f update FU 3: f*=delta;
   f=(f*C)*Dinv or (f*C)/D; f=f*(C*Dinv).
   START ST 2: 0 f=c/c1, C=f, D=0 (first D=be1); 1 NR start: f=r-form, f=tiny then term (1, c/c1), output r=f.
   EPS 3: |delta-1|<=eps, eps 1.5e-14, 1e-15, 2.2e-16. AS0 cdf=fl(fl(P*fl(xy))*r), AS1 fl(fl(P*fl(xy))/F). */
#include <math.h>
#include <stdio.h>
#define NMAX 4096
static double A[NMAX], B[NMAX], X[NMAX], Y[NMAX], P[NMAX], CDF[NMAX], RH[NMAX], RL[NMAX], EE[NMAX];
typedef struct { int w, al, e2, be, cu, df, dl, fu, st, ei; } V;
static double frac(double a, double b, double x, double y, V v, double *Fout) {
  const double epss[3] = {1.5e-14, 1e-15, 2.220446049250313e-16};
  double eps = epss[v.ei];
  double lambda = a - (a + b) * x;
  double c = lambda + 1., c0 = b / a, c1 = 1. / a + 1., yp1 = y + 1., n = 0., p = 1., s = a + 1.;
  double b0 = c / c1, f, C, D, Dinv;
  int first = 1;
  if (v.st == 0) { f = b0; C = f; D = 0.; }
  else {  /* r = 1/(b0 + K): Lentz from tiny with first partial (1, b0) */
    static const double tinys[4] = {1e-30, 1e-300, 2.2250738585072014e-308, 1e-100};
    if (v.st <= 4) {
      double tiny = tinys[v.st - 1]; f = tiny; C = f; D = 0.;
      D = b0 + 1. * D; D = 1. / D; C = b0 + 1. / C; f = f * (C * D);
    } else if (v.st == 5) { D = 1. / b0; f = D; C = 1e300; }          /* f = 1/b0 exactly, C infinite */
    else { D = 1. / b0; f = 1. / b0; C = 1. / 1e-30; f = 1e-30 * C * D; } /* left-assoc f*C*D */
  }
  for (int k = 0; k < 3000; k++) {
    double w, t, e, alpha, beta, X2, ws, delta;
    n += 1.;
    switch (v.w) { case 0: w = n * (b - n) * x; break; case 1: w = n * x * (b - n); break; default: w = n * ((b - n) * x); }
    t = n / a; e = a / s;
    alpha = v.al ? (p * (p + c0)) * ((e * e) * (w * x)) : p * (p + c0) * e * e * (w * x);
    e = v.e2 ? (t + 1.) / (c1 + 2. * t) : (t + 1.) / (c1 + t + t);
    X2 = c + n * yp1; ws = w / s;
    switch (v.be) { case 0: beta = n + ws + e * X2; break; case 1: beta = ws + (n + e * X2); break; default: beta = n + (ws + e * X2); }
    p = t + 1.; s += 2.;
    if (v.st == 1) { alpha = -(-alpha); }
    /* C */
    switch (v.cu) { case 0: C = beta + alpha / C; break; case 1: C = fma(alpha, 1. / C, beta); break;
                    case 2: C = beta + alpha * (1. / C); break; default: C = fma(alpha, 1. / C, beta); }
    /* D */
    if (v.st == 0 && first) {  /* D_1 = beta_1 (B_1/B_0); modified-Lentz forms store 1/D */
      if (v.df <= 1) D = 1. / beta; else D = beta;
      first = 0;
    } else {
      switch (v.df) { case 0: D = 1. / (beta + alpha * D); break; case 1: D = 1. / fma(alpha, D, beta); break;
                      case 2: D = beta + alpha / D; break; case 3: D = fma(alpha, 1. / D, beta); break;
                      case 4: D = beta + alpha * (1. / D); break; default: D = fma(alpha, 1. / D, beta); }
    }
    Dinv = v.df <= 1 ? D : 1. / D;
    if (v.dl == 0 || v.df <= 1) delta = C * Dinv; else delta = C / D;
    switch (v.fu) { case 0: f *= delta; break;
                    case 1: f = (v.df <= 1 || v.dl == 0) ? (f * C) * Dinv : (f * C) / D; break;
                    default: f = f * delta; }
    if (fabs(delta - 1.) <= eps) break;
  }
  if (v.st == 0) { *Fout = f; return 1. / f; }
  *Fout = 1. / f; return f;
}
int main(void) {
  int N; if (scanf("%d", &N) != 1 || N > NMAX) return 1;
  for (int i = 0; i < N; i++)
    if (scanf("%la %la %la %la %la %la %la %la %la", &A[i], &B[i], &X[i], &Y[i], &P[i], &CDF[i], &RH[i], &RL[i], &EE[i]) != 9) return 1;
  V v;
  for (v.w = 0; v.w < 3; v.w++) for (v.al = 0; v.al < 2; v.al++) for (v.e2 = 0; v.e2 < 2; v.e2++) for (v.be = 0; v.be < 3; v.be++)
  for (v.cu = 0; v.cu < 3; v.cu++) for (v.df = 0; v.df < 2; v.df++) for (v.dl = 0; v.dl < 1; v.dl++) for (v.fu = 0; v.fu < 2; v.fu++)
  for (v.st = 1; v.st < 7; v.st++) for (v.ei = 0; v.ei < 3; v.ei++) {
    if (v.df <= 1 && v.dl) continue;
    double sm = 0, sm2 = 0, se = 0, se2 = 0, sxy = 0; int e0 = 0, e1 = 0;
    for (int i = 0; i < N; i++) {
      double F, r = frac(A[i], B[i], X[i], Y[i], v, &F);
      double xy = X[i] * Y[i];
      e0 += (P[i] * xy) * r == CDF[i]; e1 += (P[i] * xy) / F == CDF[i];
      double err = ((r - RH[i]) - RL[i]) / RH[i] * 9007199254740992.0;
      sm += err; sm2 += err * err; se += EE[i]; se2 += EE[i] * EE[i]; sxy += err * EE[i];
    }
    double mm = sm / N, sd = sqrt(sm2 / N - mm * mm), me = se / N, sde = sqrt(se2 / N - me * me);
    printf("W%d AL%d E%d BE%d CU%d DF%d DL%d FU%d ST%d EPS%d %d %d %.4f %.3f %.3f\n", v.w, v.al, v.e2, v.be, v.cu, v.df, v.dl,
           v.fu, v.st, v.ei, e0, e1, (sxy / N - mm * me) / (sd * sde), mm, sd);
  }
  return 0;
}
