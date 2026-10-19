// T-1941 W2: exact-match count for the GAMMALN base kernel y = fl((x-2)*fl(N/D)), raw-x Horner, coefficients
// HIGHEST degree first, every op fl53(fl64). Build: gcc -O2 -mfpmath=387 -fPIC -shared -o libpoly.so poly.c
typedef long double LD;
static double st(LD v){volatile double d=(double)v;return d;}
int gcount(const double*p,int np,const double*q,int nq,const double*x,const double*Y,int N){
 int cnt=0;
 for(int k=0;k<N;k++){double v=x[k],n=p[0],d=q[0];
  for(int i=1;i<np;i++){n=st((LD)n*v);n=st((LD)n+p[i]);}
  for(int i=1;i<nq;i++){d=st((LD)d*v);d=st((LD)d+q[i]);}
  double s=st((LD)n/d); double m=st((LD)v-2.0); cnt+= st((LD)m*s)==Y[k];}
 return cnt;}
