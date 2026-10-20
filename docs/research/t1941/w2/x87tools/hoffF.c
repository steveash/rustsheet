#include <math.h>
typedef long double LD;
static double sd(LD v){volatile double d=(double)v;return d;}
static double ulpd(double v){double a=fabs(v);return nextafter(a,INFINITY)-a;}
/* as hoff.c, plus a fused product at one stage per chain (P left 80-bit at stage fN / fD) */
static double chain(const double*c,int n,double x,const double*off,int kh,int fuse){double s=c[0];
  for(int i=1;i<n;i++){LD P=(LD)s*x; if(i!=fuse) P=(LD)sd(P); LD v=P+(LD)c[i]; if(i<=kh && off[i]!=0){double b=sd(v); v+= (LD)off[i]*ulpd(b);} s=sd(v);} return s;}
int hcount(const double*p,int np,const double*q,int nq,const double*offN,int kN,const double*offD,int kD,const double*x,const double*Y,int N,int fN,int fD){
  int c=0; for(int k=0;k<N;k++){double n=chain(p,np,x[k],offN,kN,fN),d=chain(q,nq,x[k],offD,kD,fD); double t=sd((LD)x[k]-2.0); c+= sd((LD)t*sd((LD)n/d))==Y[k];} return c;}
