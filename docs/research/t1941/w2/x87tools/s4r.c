#include <stddef.h>
#include <math.h>
typedef long double LD;
static double sd(LD v){volatile double d=(double)v;return d;}
/* stage reader: late stored raw-x Horner from s_kN (s_k includes c[k]) / d_kD; offsets a in [-A,A], b in [-B,B] double ulps */
static double chainst(const double*c,int upto,double x){double s=c[0];for(int i=1;i<=upto;i++){s=sd((LD)s*x);s=sd((LD)s+c[i]);}return s;}
static double late(const double*c,int n,int from,double s,double x){for(int i=from+1;i<n;i++){s=sd((LD)s*x);s=sd((LD)s+c[i]);}return s;}
void s4read(const double*p,int np,const double*q,int nq,int kN,int kD,const double*x,const double*Y,int N,int A,int B,unsigned char*out){
  int W=(2*A+1)*(2*B+1);
  for(int k=0;k<N;k++){double sN=chainst(p,kN,x[k]),sD=chainst(q,kD,x[k]);
    double t=sd((LD)x[k]-2.0);
    for(int a=-A;a<=A;a++){double n0=sN; if(a>0) for(int j=0;j<a;j++) n0=nextafter(n0,INFINITY); else for(int j=0;j<-a;j++) n0=nextafter(n0,-INFINITY);
      double n=late(p,np,kN,n0,x[k]);
      for(int b=-B;b<=B;b++){double d0=sD; if(b>0) for(int j=0;j<b;j++) d0=nextafter(d0,INFINITY); else for(int j=0;j<-b;j++) d0=nextafter(d0,-INFINITY);
        double d=late(q,nq,kD,d0,x[k]); double r=sd((LD)n/d); double y=sd((LD)t*r);
        out[(size_t)k*W+(a+A)*(2*B+1)+(b+B)]= (y==Y[k]);}}}}
