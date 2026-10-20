// res = fl((x-1)*fl(N/D)); opt: cc added at stage L (form 0: into the sum, 1: into the product), caller compensates c[L-1] -= cc;
// mP/mS bitmasks of low-first indices whose product/sum is kept 80-bit. gcc -O2 -mfpmath=387 -fopenmp -shared -fPIC -o libk3s.so k3s.c
typedef long double L;
static inline double st(L v){ volatile double d=(double)v; return d; }
typedef struct { int L; int form; double cc; unsigned mP, mS; } opt;
static inline L chain(const double* c,int nc,double x,const opt* o){
    L s=c[nc-1];
    for(int i=nc-2;i>=0;i--){
        L P=(L)s*(L)x; if(i==o->L && o->form==1) P+=(L)o->cc/(L)x;
        if(!((o->mP>>i)&1)) P=st(P);
        L v=P+(L)c[i]; if(i==o->L && o->form==0) v+=(L)o->cc/(L)x;
        s = ((o->mS>>i)&1) ? v : (L)st(v);
    }
    return s;
}
static inline double res1(double x,const double*p,int np,const double*q,int nq,const opt* oN,const opt* oD){
    L N=chain(p,np,x,oN), D=chain(q,nq,x,oD); double A=st(N/D); double t=x-1.0; return st((L)t*(L)A);
}
long score(const double* x,const double* b,long n,const double* p,int np,const double* q,int nq,
           int LN,int fN,double cN,unsigned mPN,unsigned mSN,int LD,int fD,double cD,unsigned mPD,unsigned mSD){
    opt oN={LN,fN,cN,mPN,mSN}, oD={LD,fD,cD,mPD,mSD}; long c=0;
    #pragma omp parallel for reduction(+:c)
    for(long k=0;k<n;k++) c+=(res1(x[k],p,np,q,nq,&oN,&oD)==b[k]);
    return c;
}
void resv(const double* x,long n,const double* p,int np,const double* q,int nq,
          int LN,int fN,double cN,unsigned mPN,unsigned mSN,int LD,int fD,double cD,unsigned mPD,unsigned mSD,double* out){
    opt oN={LN,fN,cN,mPN,mSN}, oD={LD,fD,cD,mPD,mSD};
    #pragma omp parallel for
    for(long k=0;k<n;k++) out[k]=res1(x[k],p,np,q,nq,&oN,&oD);
}
