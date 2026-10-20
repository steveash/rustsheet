/* K1 stored Horner with hidden terms at every sum: sum j adds (c_j + e_j/y - e_{j+1}) formed in extended, one store.
   e has n+1 entries. Build: gcc -O2 -shared -fPIC -o libgsimh.so gsimh.c */
#include <math.h>
typedef long double L;
static inline L st(L v){ volatile double t=(double)v; return (L)t; }
static L chain(int n,const double*c,const double*e,L y){
  L s=c[n];
  for(int j=n-1;j>=0;j--){ L p=st(s*y); L a=(L)c[j]+(L)e[j]/y-(L)e[j+1]; s=st(p+a); }
  return s;
}
long gcounth(int n,const double*N,const double*D,const double*eN,const double*eD,long m,const double*y,const double*t){
  long c=0; for(long i=0;i<m;i++){ L a=chain(n,N,eN,(L)y[i]), b=chain(n,D,eD,(L)y[i]); volatile double q=(double)(a/b); c+=(q==t[i]); } return c;
}
void gsimh(int n,const double*N,const double*D,const double*eN,const double*eD,long m,const double*y,double*o){
  for(long i=0;i<m;i++){ L a=chain(n,N,eN,(L)y[i]), b=chain(n,D,eD,(L)y[i]); volatile double q=(double)(a/b); o[i]=q; }
}
