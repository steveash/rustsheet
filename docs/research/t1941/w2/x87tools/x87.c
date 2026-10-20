#include <math.h>
typedef long double LD;
static LD sd(LD v){volatile double d=(double)v;return d;}
/* x87 emulation: every op in 80-bit; op id (N: 2(i-1)+{0 mul,1 add}, D: 100+..., 200 div) is stored to double unless keep[id]=1 */
static double yk(const double*p,int np,const double*q,int nq,double x,const unsigned char*keep){
  LD X=x,n=p[0],d=q[0];
  for(int i=1;i<np;i++){int o=2*(i-1); n=n*X; if(!keep[o]) n=sd(n); n=n+(LD)p[i]; if(!keep[o+1]) n=sd(n);}
  for(int i=1;i<nq;i++){int o=100+2*(i-1); d=d*X; if(!keep[o]) d=sd(d); d=d+(LD)q[i]; if(!keep[o+1]) d=sd(d);}
  LD s=n/d; if(!keep[200]) s=sd(s); LD m=sd(X-2.0L); return (double)sd(m*s);}
int xcount(const double*p,int np,const double*q,int nq,const double*x,const double*Y,int N,const unsigned char*keep){
  int c=0; for(int k=0;k<N;k++) c+= yk(p,np,q,nq,x[k],keep)==Y[k]; return c;}
void xsim(const double*p,int np,const double*q,int nq,const double*x,int N,double*y,const unsigned char*keep){
  for(int k=0;k<N;k++) y[k]=yk(p,np,q,nq,x[k],keep);}
/* smooth surrogate: -(sum of min(|y-Y|/ulp(Y), cap)) scaled; higher is better */
int xl1(const double*p,int np,const double*q,int nq,const double*x,const double*Y,int N,const unsigned char*keep,int cap){
  long long s=0; for(int k=0;k<N;k++){double y=yk(p,np,q,nq,x[k],keep); double u=nextafter(fabs(Y[k]),INFINITY)-fabs(Y[k]);
    double e=fabs(y-Y[k])/u; long long c=(e>cap)?cap:(long long)(e+0.5); s+=c*1000 - (e==0?1:0);}
  return (int)(-s);}
