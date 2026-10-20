#include <math.h>
typedef long double LD;
static double sd(LD v){volatile double d=(double)v;return d;}
/* per-cluster smooth-offset scan. Early value E (80-bit Horner, double consts) of N through stage kN, D through kD;
   hand-off s = fl53(E + dN*ulp(fl(E))), d likewise; late stored raw-x Horner; y = fl(t*fl(n/d)).
   counts[iN*nD+iD] = #points matching, for dN = (iN - GN)/R, dD = (iD - GD)/R. */
static LD early(const double*c,int k,double x){LD s=c[0];for(int i=1;i<=k;i++){s=s*(LD)x;s=s+(LD)c[i];}return s;}
static double late(const double*c,int n,int from,double s,double x){for(int i=from+1;i<n;i++){s=sd((LD)s*x);s=sd((LD)s+c[i]);}return s;}
void dgrid(const double*p,int np,const double*q,int nq,int kN,int kD,const double*x,const double*Y,int N,int GN,int GD,int R,int*counts){
  int nN=2*GN+1,nD=2*GD+1;
  for(int k=0;k<N;k++){LD EN=early(p,kN,x[k]),ED=early(q,kD,x[k]); double uN=nextafter(fabs((double)EN),INFINITY)-fabs((double)EN), uD=nextafter(fabs((double)ED),INFINITY)-fabs((double)ED);
    double t=sd((LD)x[k]-2.0);
    double lastsN=NAN, nN_=0;
    for(int iN=0;iN<nN;iN++){double sN=sd(EN+(LD)(iN-GN)/R*uN); double n=(sN==lastsN)?nN_:late(p,np,kN,sN,x[k]); lastsN=sN; nN_=n;
      double lastsD=NAN, dv=0;
      for(int iD=0;iD<nD;iD++){double sD=sd(ED+(LD)(iD-GD)/R*uD); double d=(sD==lastsD)?dv:late(q,nq,kD,sD,x[k]); lastsD=sD; dv=d;
        counts[iN*nD+iD]+= (sd((LD)t*sd((LD)n/d))==Y[k]);}}}}
