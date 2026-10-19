// T-1941 W5 part 5: x87 simulation of the K3 kernel with an injected shift at one rounding site.
// Site: chain (0 = N, 1 = D), stage j (P_j = fl(S_{j+1}*x), S_j = fl(P_j + c_j)), kind (0 = P, 1 = S),
// or chain 2 = the division A, chain 3 = the final product. The exact (64-bit) value at that site is shifted by
// phi * ulp53(stored value) before the 53-bit store. Build: gcc -O2 -mfpmath=387 -fopenmp -shared -fPIC -o libinj.so inj.c
#include <math.h>
typedef long double L;
static inline double st(L v){ volatile double d=(double)v; return d; }
static inline L sh(L v,double phi){ double s=st(v); double u=nextafter(fabs(s),INFINITY)-fabs(s); return v+(L)phi*(L)u; }
static inline double horner(const double* c,int nc,double x,int inj,int j0,int kind,double phi){
    double s=c[nc-1];
    for(int j=nc-2;j>=0;j--){ L p=(L)s*(L)x; if(inj&&j==j0&&kind==0) p=sh(p,phi); double w=st(p);
        L q=(L)w+(L)c[j]; if(inj&&j==j0&&kind==1) q=sh(q,phi); s=st(q); }
    return s;
}
static inline double res1(double x,const double*p,int np,const double*q,int nq,int ch,int j0,int kind,double phi){
    double N=horner(p,np,x,ch==0,j0,kind,phi), D=horner(q,nq,x,ch==1,j0,kind,phi);
    L a=(L)N/(L)D; if(ch==2) a=sh(a,phi); double A=st(a); double t=x-1.0; L r=(L)t*(L)A; if(ch==3) r=sh(r,phi); return st(r);
}
// out[m*n] for m phases
void res_inj(const double* x,long n,const double*p,int np,const double*q,int nq,int ch,int j0,int kind,const double* phis,int m,double* out){
    #pragma omp parallel for collapse(2)
    for(int i=0;i<m;i++) for(long k=0;k<n;k++) out[(long)i*n+k]=res1(x[k],p,np,q,nq,ch,j0,kind,phis[i]);
}
