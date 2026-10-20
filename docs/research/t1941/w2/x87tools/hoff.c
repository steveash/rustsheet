#include <math.h>
typedef long double LD;
static double sd(LD v){volatile double d=(double)v;return d;}
static double ulpd(double v){double a=fabs(v);return nextafter(a,INFINITY)-a;}
/* stored raw-x Horner; at each head sum i in 1..kh (sum s_i = fl(P_i + c_i)), add off[i]*ulp(base s_i) before rounding */
static double chain(const double*c,int n,double x,const double*off,int kh){double s=c[0];
  for(int i=1;i<n;i++){double P=sd((LD)s*x); LD v=(LD)P+(LD)c[i]; if(i<=kh && off[i]!=0){double b=sd(v); v+= (LD)off[i]*ulpd(b);} s=sd(v);} return s;}
int hcount(const double*p,int np,const double*q,int nq,const double*offN,int kN,const double*offD,int kD,const double*x,const double*Y,int N){
  int c=0; for(int k=0;k<N;k++){double n=chain(p,np,x[k],offN,kN),d=chain(q,nq,x[k],offD,kD); double t=sd((LD)x[k]-2.0); c+= sd((LD)t*sd((LD)n/d))==Y[k];} return c;}
