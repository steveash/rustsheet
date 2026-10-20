/* K1 simulator with keep masks: bit (2k) = product k unstored, bit (2k+1) = sum k unstored (k=0 top); per chain mask.
   build: gcc -O2 -shared -fPIC -o libgsimk.so gsimk.c */
#include <math.h>
typedef long double L;
static inline L st(L v){ volatile double t=(double)v; return (L)t; }
static L chain(int n,const double*c,L y,unsigned mask){
  L s=c[n];
  for(int k=0;k<n;k++){ L p=s*y; if(!(mask>>(2*k)&1)) p=st(p); s=p+(L)c[n-1-k]; if(!(mask>>(2*k+1)&1)) s=st(s); }
  return s;
}
long gcountk(int n,const double*N,const double*D,unsigned mN,unsigned mD,long m,const double*y,const double*t){
  long c=0; for(long i=0;i<m;i++){ L a=chain(n,N,(L)y[i],mN), b=chain(n,D,(L)y[i],mD); volatile double q=(double)(a/b); c+=(q==t[i]); } return c;
}
void gsimk(int n,const double*N,const double*D,unsigned mN,unsigned mD,long m,const double*y,double*o){
  for(long i=0;i<m;i++){ L a=chain(n,N,(L)y[i],mN), b=chain(n,D,(L)y[i],mD); volatile double q=(double)(a/b); o[i]=q; }
}
