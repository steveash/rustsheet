/* Copied by T-1916 from branch agent/research/T-1778-kernel-map at 59c2c3724a3c (path docs/research/t1778/round2/sc.c), unchanged below this line. */
/* sc.c: exact scorer for erf = fl53( div(co, W) ), co = fl53(fl64(x*N)), N = fl(P(z)), W = fl(Q(z)).
 * usage: sc COEF MODE FILE...   COEF: "P n c0..cn\nQ m d0..dm" (decimal or hex, parsed as long double)
 * MODE: 4 digits  z h d c
 *   z: 0 z=fl53(x*x) [via 80-bit then store: same as double since exact product fits 106 bits -> double rounding possible]
 *      1 z = (double)(x*x) plain SSE double ; 2 z kept in 80-bit (LD x*x)
 *   h: 0 Horner in 80-bit with coefficients as long double; 1 coefficients rounded to double, Horner 80-bit, result stored
 *      2 coefficients double, Horner in plain double (each op rounded to 53)
 *      3 coefficients double, Horner in 80-bit but each step stored (double-rounded)
 *      4 coefficients double, FMA Horner in double
 *   d: 0 80-bit divide then round53 ; 1 double divide
 *   c: 0 co = fl53(fl64(x*N)) ; 1 co = double x*N
 * env VERB=1 prints per-run; BANDS printed per file. Only |x|<0.5 scored. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
typedef long double LD;
static LD Pc[32], Qc[32]; static int np, nq; static double Pd[32], Qd[32];
static int zm, hm, dm, cm;
static double st(LD v) { volatile double d = (double)v; return d; }
static double evalpoly(int n, LD *c, double *cd, double zd, LD zl) {
  if (hm == 0) { LD s = c[n]; LD z = (zm == 2) ? zl : (LD)zd; for (int i = n - 1; i >= 0; i--) s = s * z + c[i]; return st(s); }
  if (hm == 1) { LD s = cd[n]; LD z = (zm == 2) ? zl : (LD)zd; for (int i = n - 1; i >= 0; i--) s = s * z + (LD)cd[i]; return st(s); }
  if (hm == 2) { volatile double s = cd[n]; for (int i = n - 1; i >= 0; i--) { volatile double t = s * zd; s = t + cd[i]; } return s; }
  if (hm == 3) { double s = cd[n]; for (int i = n - 1; i >= 0; i--) { volatile LD t = (LD)s * (LD)zd; double tt = st(t); volatile LD u = (LD)tt + (LD)cd[i]; s = st(u); } return s; }
  if (hm == 6) { double s = cd[n]; for (int i = n - 1; i >= 0; i--) { volatile LD t = (LD)s * (LD)zd + (LD)cd[i]; s = st(t); } return s; }
  if (hm == 4) { double s = cd[n]; for (int i = n - 1; i >= 0; i--) s = fma(s, zd, cd[i]); return s; }
  return 0;
}
static double model(double x) {
  double zd; LD zl = (LD)x * (LD)x;
  if (zm == 0) zd = st(zl); else { volatile double t = x * x; zd = t; }
  double N = evalpoly(np, Pc, Pd, zd, zl), W = evalpoly(nq, Qc, Qd, zd, zl);
  double co; if (cm == 0) { volatile LD p = (LD)x * (LD)N; co = st(p); } else { volatile double p = x * N; co = p; }
  if (dm == 0) { volatile LD q = (LD)co / (LD)W; return st(q); }
  volatile double q = co / W; return q;
}
static const double BE[] = {0, 1e-16, 1e-12, 1e-8, 1e-6, 1e-4, 1e-3, 4.2e-3, 1.12e-2, 3.3e-2, 0.1, 0.26};
#define NB 11
int main(int argc, char **argv) {
  FILE *cf = fopen(argv[1], "r"); char tag[8];
  if (!cf) { fprintf(stderr, "no coef\n"); return 1; }
  while (fscanf(cf, "%7s", tag) == 1) { int n; if (fscanf(cf, "%d", &n) != 1) return 1; LD *c = tag[0] == 'P' ? Pc : Qc; double *cd = tag[0] == 'P' ? Pd : Qd;
    if (tag[0] == 'P') np = n; else nq = n; for (int i = 0; i <= n; i++) { char s[80]; if (fscanf(cf, "%79s", s) != 1) return 1; c[i] = strtold(s, 0); cd[i] = (double)c[i]; } }
  const char *m = argv[2]; zm = m[0] - '0'; hm = m[1] - '0'; dm = m[2] - '0'; cm = m[3] - '0';
  int verb = getenv("VERB") ? atoi(getenv("VERB")) : 0; long GM = 0, GT = 0;
  for (int fi = 3; fi < argc; fi++) {
    FILE *f = fopen(argv[fi], "r"); if (!f) continue; char a[64], b[64]; int id, last = -999999; long rm = 0, rt = 0; double rz = 0;
    long bm[NB] = {0}, bt[NB] = {0};
    while (fscanf(f, "%d %63s %63s", &id, a, b) == 3) {
      double x = strtod(a, 0), y = strtod(b, 0); if (fabs(x) >= 0.5) continue;
      if (id != last) { if (verb && rt) printf("  run %d z=%.4e %ld/%ld\n", last, rz, rm, rt); rm = rt = 0; last = id; }
      double z = x * x; int ok = model(x) == y; rm += ok; rt++; rz = z;
      int k = 0; while (k < NB - 1 && z >= BE[k + 1]) k++; bm[k] += ok; bt[k]++;
      if (verb > 1 && !ok) printf("    MISS %d %a %a got %a\n", id, x, y, model(x));
    }
    if (verb && rt) printf("  run %d z=%.4e %ld/%ld\n", last, rz, rm, rt);
    long M = 0, T = 0; printf("%s:", argv[fi]);
    for (int k = 0; k < NB; k++) if (bt[k]) { printf(" [%.0e,%.0e) %ld/%ld", BE[k], BE[k + 1], bm[k], bt[k]); M += bm[k]; T += bt[k]; }
    printf("  TOTAL %ld/%ld\n", M, T); GM += M; GT += T; fclose(f);
  }
  printf("GRAND %ld/%ld\n", GM, GT); return 0;
}
