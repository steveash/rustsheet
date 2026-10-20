/* plain stored Horner with no-op shifts: sum j: +o_j then sum j-1: -o_j*y ; product P_j (feeding sum j): +q_j then sum j: -q_j. All in extended. */
typedef long double L;
static inline L st(L v){ volatile double t=(double)v; return (L)t; }
static L chain(int n,const double*c,const double*o,const double*q,L y){
  L s=c[n];
  for(int j=n-1;j>=0;j--){ L p=st(s*y+(L)q[j]); L a=(L)c[j]-(L)q[j]+(L)o[j]-(L)o[j+1]*y; s=st(p+a); }
  return s; }
long gcountn(int n,const double*N,const double*D,const double*oN,const double*oD,const double*qN,const double*qD,long m,const double*y,const double*t){
  long c=0; for(long i=0;i<m;i++){ L a=chain(n,N,oN,qN,(L)y[i]), b=chain(n,D,oD,qD,(L)y[i]); volatile double r=(double)(a/b); c+=(r==t[i]); } return c; }
