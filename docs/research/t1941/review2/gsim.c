/* T-1941 review2: exact x87-stored simulator of K1's G = fl(N(y)/D(y)), raw-y Horner, every op fl53(fl64), with
   per-stage NUDGES: pn[k]/sn[k] (k = 0..n-1, k = 0 is the top product/sum) are fractions of the ulp of the double
   nearest the pre-rounding value, added to that value before it is stored. A nudge is a local coordinate for the
   sub-ulp part of the upstream stage value at a run (see HANDOFF section 11). Build:
   gcc -O2 -shared -fPIC -o libgsim.so gsim.c   (long double is x87 80-bit on x86-64 regardless of -mfpmath) */
#include <math.h>
typedef long double L;
static inline double st(L v){ volatile double t=(double)v; return t; }
static inline double ulp_of(L v){ double d=fabs((double)v); return nextafter(d,INFINITY)-d; }
static double chain(int n,const double*c,double y,const double*pn,const double*sn,double*trace){
  /* c ascending, n = degree; stages k=0..n-1 */
  L s=c[n];
  for(int k=0;k<n;k++){
    L pe=s*(L)y; if(pn&&pn[k]!=0) pe+= (L)pn[k]*(L)ulp_of(pe); double p=st(pe);
    L se=(L)p+(L)c[n-1-k]; if(sn&&sn[k]!=0) se+= (L)sn[k]*(L)ulp_of(se); s=st(se);
    if(trace){ trace[2*k]=p; trace[2*k+1]=(double)s; }
  }
  return (double)s;
}
void gsim(int n,const double*N,const double*D,const double*pn,const double*sn,const double*pd,const double*sd,
          long m,const double*y,double*out){
  for(long i=0;i<m;i++){ L a=chain(n,N,y[i],pn,sn,0), b=chain(n,D,y[i],pd,sd,0); out[i]=st(a/b); }
}
long gcount(int n,const double*N,const double*D,const double*pn,const double*sn,const double*pd,const double*sd,
            long m,const double*y,const double*target){
  long c=0;
  for(long i=0;i<m;i++){ L a=chain(n,N,y[i],pn,sn,0), b=chain(n,D,y[i],pd,sd,0); c+=(st(a/b)==target[i]); }
  return c;
}
void gtrace(int n,const double*c,long m,const double*y,const double*pn,const double*sn,double*trace){
  for(long i=0;i<m;i++) chain(n,c,y[i],pn,sn,trace+2*n*i);
}
