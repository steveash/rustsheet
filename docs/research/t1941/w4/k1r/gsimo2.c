/* plain stored Horner with constant offsets at products (qN/qD) and sums (oN/oD), extended, uncompensated */
typedef long double L;
static inline L st(L v){ volatile double t=(double)v; return (L)t; }
static L chain(int n,const double*c,const double*o,const double*q,L y){ L s=c[n]; for(int j=n-1;j>=0;j--){ L p=st(s*y+(L)q[j]); s=st(p+(L)c[j]+(L)o[j]); } return s; }
long gcounto2(int n,const double*N,const double*D,const double*oN,const double*oD,const double*qN,const double*qD,long m,const double*y,const double*t){
  long c=0; for(long i=0;i<m;i++){ L a=chain(n,N,oN,qN,(L)y[i]), b=chain(n,D,oD,qD,(L)y[i]); volatile double r=(double)(a/b); c+=(r==t[i]); } return c; }
