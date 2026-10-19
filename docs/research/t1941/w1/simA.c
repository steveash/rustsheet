// T-1941 W1: exact x87 simulation A = RN(N(x)/D(x)), Horner in raw x, every op fl53(fl64(.)).
// Build: gcc -O1 -mfpmath=387 -shared -fPIC -o libsimA.so simA.c (x86-64, default extended precision control).
typedef long double L;
static double st(L v){ volatile double d=(double)v; return d; }
static double horner(const double* c,int nc,double x){
    double s=c[nc-1];
    for(int i=nc-2;i>=0;i--){ double w=st((L)s*(L)x); s=st((L)w+(L)c[i]); }
    return s;
}
void simA(const double* x,int n,const double* p,int np,const double* q,int nq,double* out){
    for(int k=0;k<n;k++){ double N=horner(p,np,x[k]); double D=horner(q,nq,x[k]); out[k]=st((L)N/(L)D); }
}
void simA_batch(const double* x,int n,const double* P,int np,const double* Q,int nq,int m,double* out){
    for(int j=0;j<m;j++) simA(x,n,P+j*np,np,Q+j*nq,nq,out+(long)j*n);
}
