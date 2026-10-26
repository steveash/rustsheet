// T-2464 w4-fractions: continued-fraction candidates for I_x(a,b), templated over the arithmetic type.
// T = double (SSE build: every op rounded to double), Dx (x87: every op rounded to 64 bits then stored to double),
// long double (x87 extended throughout).
#pragma once
#include <cmath>
#include <cfloat>
#include <cstdio>

struct Dx {  // stored x87: fl53(fl64(a op b))
  double v;
  Dx() : v(0) {}
  Dx(double d) : v(d) {}
  static Dx st(long double r) { volatile double d = (double)r; return Dx(d); }
  friend Dx operator+(Dx a, Dx b) { return st((long double)a.v + (long double)b.v); }
  friend Dx operator-(Dx a, Dx b) { return st((long double)a.v - (long double)b.v); }
  friend Dx operator*(Dx a, Dx b) { return st((long double)a.v * (long double)b.v); }
  friend Dx operator/(Dx a, Dx b) { return st((long double)a.v / (long double)b.v); }
  friend Dx operator-(Dx a) { return Dx(-a.v); }
  Dx& operator+=(Dx b) { *this = *this + b; return *this; }
  Dx& operator-=(Dx b) { *this = *this - b; return *this; }
  Dx& operator*=(Dx b) { *this = *this * b; return *this; }
  Dx& operator/=(Dx b) { *this = *this / b; return *this; }
  friend bool operator<(Dx a, Dx b) { return a.v < b.v; }
  friend bool operator>(Dx a, Dx b) { return a.v > b.v; }
  friend bool operator<=(Dx a, Dx b) { return a.v <= b.v; }
  friend bool operator>=(Dx a, Dx b) { return a.v >= b.v; }
  friend bool operator==(Dx a, Dx b) { return a.v == b.v; }
  friend bool operator!=(Dx a, Dx b) { return a.v != b.v; }
};
inline Dx fabs(Dx a) { return Dx(std::fabs(a.v)); }
inline long double LD(Dx a) { return a.v; }
inline long double LD(double a) { return a; }
inline long double LD(long double a) { return a; }
template <class T> inline T absT(T a) { return a < T(0) ? -a : a; }

// ---- Numerical Recipes betacf (modified Lentz). eps, fpmin, maxit parameters; variant nr2 (NR in C 2nd ed.) or nr3.
template <class T> T nr_betacf(T a, T b, T x, double eps, double fpmin, int maxit, int *iters) {
  T qab = a + b, qap = a + T(1), qam = a - T(1), c = T(1), d = T(1) - qab * x / qap;
  if (absT(d) < T(fpmin)) d = T(fpmin);
  d = T(1) / d;
  T h = d;
  int m;
  for (m = 1; m <= maxit; m++) {
    T m_ = T(m), m2 = T(2 * m);
    T aa = m_ * (b - m_) * x / ((qam + m2) * (a + m2));
    d = T(1) + aa * d; if (absT(d) < T(fpmin)) d = T(fpmin);
    c = T(1) + aa / c; if (absT(c) < T(fpmin)) c = T(fpmin);
    d = T(1) / d; h *= d * c;
    aa = -(a + m_) * (qab + m_) * x / ((a + m2) * (qap + m2));
    d = T(1) + aa * d; if (absT(d) < T(fpmin)) d = T(fpmin);
    c = T(1) + aa / c; if (absT(c) < T(fpmin)) c = T(fpmin);
    d = T(1) / d;
    T del = d * c; h *= del;
    if (absT(del - T(1)) <= T(eps)) break;
  }
  if (iters) *iters = m;
  return h;  // = CF (I = prefactor * h / a)
}

// ---- Cephes incbcf / incbd (forward recurrence, big/biginv rescale, thresh 3*MACHEP)
template <class T> T cephes_frac(T x, T k1, T k2, T k3, T k4, T k5, T k6, T k7, T k8, bool second, double thresh_mult, int *iters) {
  const double MACHEP = 1.11022302462515654042e-16, big = 4.503599627370496e15, biginv = 2.22044604925031308085e-16;
  T pkm2 = T(0), qkm2 = T(1), pkm1 = T(1), qkm1 = T(1), ans = T(1), r = T(1), t, xk, pk, qk;
  T thresh = T(thresh_mult * MACHEP);
  int n = 0;
  do {
    xk = -(x * k1 * k2) / (k3 * k4);
    pk = pkm1 + pkm2 * xk; qk = qkm1 + qkm2 * xk;
    pkm2 = pkm1; pkm1 = pk; qkm2 = qkm1; qkm1 = qk;
    xk = (x * k5 * k6) / (k7 * k8);
    pk = pkm1 + pkm2 * xk; qk = qkm1 + qkm2 * xk;
    pkm2 = pkm1; pkm1 = pk; qkm2 = qkm1; qkm1 = qk;
    if (qk != T(0)) r = pk / qk;
    if (r != T(0)) { t = absT((ans - r) / r); ans = r; } else t = T(1);
    if (t < thresh) break;
    k1 += T(1); if (second) k2 -= T(1); else k2 += T(1); k3 += T(2); k4 += T(2); k5 += T(1);
    if (second) k6 += T(1); else k6 -= T(1); k7 += T(2); k8 += T(2);
    if ((absT(qk) + absT(pk)) > T(big)) { pkm2 *= T(biginv); pkm1 *= T(biginv); qkm2 *= T(biginv); qkm1 *= T(biginv); }
    if ((absT(qk) < T(biginv)) || (absT(pk) < T(biginv))) { pkm2 *= T(big); pkm1 *= T(big); qkm2 *= T(big); qkm1 *= T(big); }
  } while (++n < 300);
  if (iters) *iters = n + 1;
  return ans;
}
template <class T> T cephes_incbcf(T a, T b, T x, double tm, int *it) {
  return cephes_frac<T>(x, a, a + b, a, a + T(1), T(1), b - T(1), a + T(1), a + T(2), false, tm, it);
}
template <class T> T cephes_incbd_z(T a, T b, T z, double tm, int *it) {
  return cephes_frac<T>(z, a, b - T(1), a, a + T(1), T(1), a + b, a + T(1), a + T(2), true, tm, it);
}

// ---- LibreOffice GetBetaDist fraction (A&S 26.5.8 forward, normalized each step)
template <class T> T lo_cf(T a, T b, T x, int *iters) {
  T a1 = T(1), b1 = T(1), b2 = T(1) - (a + b) / (a + T(1)) * x, a2, fnorm, cf, cfnew = T(1), rm = T(1);
  if (b2 == T(0)) { a2 = T(0); fnorm = T(1); cf = T(1); }
  else { a2 = T(1); fnorm = T(1) / b2; cf = a2 * fnorm; }
  int n = 0;
  for (n = 0; n < 50000; n++) {
    T apl2m = a + T(2) * rm;
    T d2m = rm * (b - rm) * x / ((apl2m - T(1)) * apl2m);
    T d2m1 = -(a + rm) * (a + b + rm) * x / (apl2m * (apl2m + T(1)));
    a1 = (a2 + d2m * a1) * fnorm;
    b1 = (b2 + d2m * b1) * fnorm;
    a2 = a1 + d2m1 * a2 * fnorm;
    b2 = b1 + d2m1 * b2 * fnorm;
    bool fin = false;
    if (b2 != T(0)) { fnorm = T(1) / b2; cfnew = a2 * fnorm; fin = absT(cf - cfnew) < absT(cf) * T(DBL_EPSILON); }
    cf = cfnew; rm += T(1);
    if (fin) break;
  }
  if (iters) *iters = n + 1;
  return cf;
}

// ---- TOMS 708 bfrac fraction part r (I = brcomp * r); lambda passed in.
template <class T> T toms_bfrac_r(T a, T b, T x, T y, T lambda, double eps, int *iters) {
  T c = lambda + T(1), c0 = b / a, c1 = T(1) / a + T(1), yp1 = y + T(1), n = T(0), p = T(1), s = a + T(1),
    an = T(0), bn = T(1), anp1 = T(1), bnp1 = c / c1, r = c1 / c, r0;
  int k = 0;
  do {
    n += T(1); k++;
    T w = n * x * (b - n);
    T t = n / a, e = a / s, alpha = p * (p + c0) * e * e * (w * x), beta;
    e = (t + T(1)) / (c1 + t + t);
    beta = w / s + (n + e * (c + n * yp1));
    p = t + T(1); s += T(2);
    t = alpha * an + beta * anp1; an = anp1; anp1 = t;
    t = alpha * bn + beta * bnp1; bn = bnp1; bnp1 = t;
    r0 = r; r = anp1 / bnp1;
    if (absT(r - r0) <= T(eps) * r) break;
    an /= bnp1; bn /= bnp1; anp1 = r; bnp1 = T(1);
  } while (k < 1000);
  if (iters) *iters = k;
  return r;
}

// ---- A&S 26.5.8 backward evaluation with N levels: CF = 1/(1+d1/(1+d2/(1+...)))
template <class T> T as_backward(T a, T b, T x, int N) {
  T f = T(1);
  for (int m = N; m >= 1; m--) {
    int k = m / 2; T K = T(k), d;
    if (m % 2 == 1) d = -((a + K) * (a + b + K) * x) / ((a + T(2) * K) * (a + T(2) * K + T(1)));
    else d = (K * (b - K) * x) / ((a + T(2) * K - T(1)) * (a + T(2) * K));
    f = T(1) + d / f;
  }
  return T(1) / f;
}
