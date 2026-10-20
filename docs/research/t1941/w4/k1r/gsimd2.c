/* plain stored Horner; per-point no-op delta: sum j0 adds +dl, sum j0-1 adds -dl*y (extended), chain ch */
typedef long double L;
static inline L st(L v){ volatile double t=(double)v; return (L)t; }
static L chain(int n,const double*c,L y,int j0,L dl){
  L s=c[n];
  for(int j=n-1;j>=0;j--){ L p=st(s*y); L a=(L)c[j]; if(j==j0) a+=dl; else if(j==j0-1) a-=dl*y; s=st(p+a); }
  return s;
}
long gcountd2(int n,const double*N,const double*D,int ch,int j0,long m,const double*y,const double*dl,const double*t){
  long c=0; for(long i=0;i<m;i++){ L a=chain(n,N,(L)y[i],ch==0?j0:-9,(L)dl[i]), b=chain(n,D,(L)y[i],ch==1?j0:-9,(L)dl[i]); volatile double q=(double)(a/b); c+=(q==t[i]); } return c;
}
