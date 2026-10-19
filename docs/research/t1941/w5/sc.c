// T-1941 W5: exact x87 scorer for the GAMMALN near-1 kernel. res = fl((x-1)*fl(N(x)/D(x))), raw-x Horner,
// every op fl53(fl64(.)). Build: gcc -O2 -mfpmath=387 -fopenmp -shared -fPIC -o libsc.so sc.c
typedef long double L;
static inline double st(L v){ volatile double d=(double)v; return d; }
static inline double horner(const double* c,int nc,double x){
    double s=c[nc-1];
    for(int i=nc-2;i>=0;i--){ double w=st((L)s*(L)x); s=st((L)w+(L)c[i]); }
    return s;
}
static inline double res1(double x,const double*p,int np,const double*q,int nq){
    double N=horner(p,np,x), D=horner(q,nq,x); double A=st((L)N/(L)D); double t=x-1.0; return st((L)t*(L)A);
}
// count exact matches
long score(const double* x,const double* b,long n,const double* p,int np,const double* q,int nq){
    long c=0;
    #pragma omp parallel for reduction(+:c)
    for(long k=0;k<n;k++) c+= (res1(x[k],p,np,q,nq)==b[k]);
    return c;
}
void resv(const double* x,long n,const double* p,int np,const double* q,int nq,double* out){
    #pragma omp parallel for
    for(long k=0;k<n;k++) out[k]=res1(x[k],p,np,q,nq);
}
// batch: m candidate sets, scores out[m]
void score_batch(const double* x,const double* b,long n,const double* P,int np,const double* Q,int nq,int m,long* out){
    #pragma omp parallel for schedule(dynamic)
    for(int j=0;j<m;j++){ long c=0; const double*p=P+(long)j*np,*q=Q+(long)j*nq;
        for(long k=0;k<n;k++) c+=(res1(x[k],p,np,q,nq)==b[k]); out[j]=c; }
}
