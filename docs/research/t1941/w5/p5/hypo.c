// T-1941 W5 part 5: exact chain-top hypothesis objective.
// For each hypothesis h (coefficients c_top..c_J0 of one chain, ascending index, nc = top-J0+1 values), simulate per
// point s = c_top; for j = top-1..J0: p = RN(s*x), s = RN(p + c_j); template t += w_j(x) * (eP + eS); then add the
// product P_{J0-1} = RN(s*x) error times w_{J0-1}. Objective = sum over runs of corr(t, r)*sqrt(n).
// W is [nst][npts] with W[k] = weight of stage (J0-1+k). Build: gcc -O2 -mfpmath=387 -fopenmp -shared -fPIC -o libhypo.so hypo.c
#include <math.h>
typedef long double L;
static inline double st(L v){ volatile double d=(double)v; return d; }
void hypo_obj(const double* x,const double* r,const long* rs,int nr,long npts,const double* W,int J0,
              const double* H,int nc,int nh,int withlast,double* out){
    #pragma omp parallel for schedule(dynamic)
    for(int h=0;h<nh;h++){
        const double* c=H+(long)h*nc; double tot=0;   // c[0] = c_J0 ... c[nc-1] = c_top
        for(int i=0;i<nr;i++){
            double s1=0,s2=0,s3=0,s4=0; long a=rs[i],b=rs[i+1]; long n=b-a;
            for(long p=a;p<b;p++){
                double xv=x[p]; double s=c[nc-1]; L t=0;
                for(int k=nc-2;k>=0;k--){           // stage j = J0+k
                    L pe=(L)s*(L)xv; double P=st(pe); L se=(L)P+(L)c[k]; s=st(se);
                    t+=(L)W[(long)(k+1)*npts+p]*(((L)P-pe)+((L)s-se));
                }
                if(withlast){ L pe=(L)s*(L)xv; double P=st(pe); t+=(L)W[p]*((L)P-pe); }   // product P_{J0-1}
                double tv=(double)t; s1+=tv; s2+=tv*tv; s3+=tv*r[p]; s4+=r[p]*r[p];
            }
            double var=s2-s1*s1/n; if(var>0&&s4>0) tot+=s3/sqrt(var)/sqrt(s4)*sqrt((double)n);
        }
        out[h]=tot;
    }
}
