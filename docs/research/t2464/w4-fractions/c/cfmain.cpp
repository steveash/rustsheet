// Reads lines "a x b y" (hex doubles: a b x y) and prints, per line, every CF variant's value (%La) in mode T.
#include <initializer_list>
#include "cf.hpp"
#ifndef MODE
#define MODE 0
#endif
#if MODE == 0
typedef double T;  // SSE
#elif MODE == 1
typedef Dx T;      // x87 stored
#else
typedef long double T;  // x87 extended
#endif
int main() {
  double a, b, x, y;
  while (scanf("%la %la %la %la", &a, &b, &x, &y) == 4) {
    T A(a), B(b), X(x), Y(y);
    int it;
    const double nreps[] = {3e-16, 2.220446049250313e-16, 1e-15, 1e-14, 0.0};
    for (double e : nreps) printf("%La ", LD(nr_betacf<T>(A, B, X, e, 1e-300, (e == 0 ? 300 : 1000), &it)));
    printf("%La ", LD(cephes_incbcf<T>(A, B, X, 3.0, &it)));
    printf("%La ", LD(cephes_incbcf<T>(A, B, X, 1.0, &it)));
    T Z = X / Y;
    printf("%La ", LD(cephes_incbd_z<T>(A, B, Z, 3.0, &it)));
    printf("%La ", LD(lo_cf<T>(A, B, X, &it)));
    T lam = A - (A + B) * X;
    T lamy = (A + B) * Y - B;
    printf("%La ", LD(toms_bfrac_r<T>(A, B, X, Y, lam, 1.5e-14, &it)));
    printf("%La ", LD(toms_bfrac_r<T>(A, B, X, Y, lam, 2.2e-16, &it)));
    printf("%La ", LD(toms_bfrac_r<T>(A, B, X, Y, lamy, 1.5e-14, &it)));
    for (int N : {30, 60, 120, 240}) printf("%La ", LD(as_backward<T>(A, B, X, N)));
    printf("\n");
  }
}
