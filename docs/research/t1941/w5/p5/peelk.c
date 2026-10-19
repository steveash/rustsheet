// T-1941 W5 part 5: exact two-site hypothesis objective for top-down peeling.
// For each integer move k of c_J (upstream exact): S = RN(P_J + c_J + k*uc), P' = RN(S*x); template per point
// t = wS*eS + wP*eP (eS, eP = stored - exact, absolute); objective = sum over runs of the normalised correlation of t
// with the residual r (r already zero-mean per run). Build: gcc -O2 -mfpmath=387 -fopenmp -shared -fPIC -o libpeelk.so peelk.c
#include <math.h>
typedef long double L;
static inline double st(L v){ volatile double d=(double)v; return d; }
void peel_obj(const double* PJ,const double* x,const double* wS,const double* wP,const double* r,const long* rs,int nr,
              double cJ,double uc,const long* ks,int nk,double* out){
    #pragma omp parallel for schedule(dynamic)
    for(int q=0;q<nk;q++){
        L cc=(L)cJ+(L)ks[q]*(L)uc; double tot=0;
        for(int i=0;i<nr;i++){
            double st_=0,stt=0,str=0,srr=0; long a=rs[i],b=rs[i+1]; long n=b-a;
            for(long p=a;p<b;p++){
                L se=(L)PJ[p]+cc; double S=st(se); L eS=(L)S-se;
                L pe=(L)S*(L)x[p]; double P=st(pe); L eP=(L)P-pe;
                double t=(double)(wS[p]*eS+wP[p]*eP);
                st_+=t; stt+=t*t; str+=t*r[p]; srr+=r[p]*r[p];
            }
            double var=stt-st_*st_/n; if(var>0&&srr>0) tot+=str/sqrt(var)/sqrt(srr)*sqrt((double)n);
        }
        out[q]=tot;
    }
}
