// T-1941 W2: as poly.c, with a constant integer ulp offset added after each stored partial sum (stage-offset probe).
typedef long double LD;
static double st(LD v){volatile double d=(double)v;return d;}
static double nd(double v,long k){ union{double d; long long i;} u; u.d=v; if(v==0) return v; u.i+= (v>=0? k : -k); return u.d;}
int gcoff(const double*p,int np,const double*q,int nq,const double*x,const double*Y,int N,const int*oN,const int*oD){
 int cnt=0;
 for(int k=0;k<N;k++){double v=x[k],n=p[0],d=q[0];
  for(int i=1;i<np;i++){n=st((LD)n*v);n=st((LD)n+p[i]); n=nd(n,oN[i-1]);}
  for(int i=1;i<nq;i++){d=st((LD)d*v);d=st((LD)d+q[i]); d=nd(d,oD[i-1]);}
  double s=st((LD)n/d); double m=st((LD)v-2.0); cnt+= st((LD)m*s)==Y[k];}
 return cnt;}
