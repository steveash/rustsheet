// Find doubles a whose exact 2^(RN64(a*L2E64)) lies within |delta|<DMAX (units G = 2^-11 ulp53) of a
// 53-bit rounding midpoint. mode "cluster f0 n count": scan consecutive doubles from a0=(n+f0)/L2E.
// mode "rand seed count": log-uniform |a| in [2^-4,709], random sign.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <quadmath.h>
static const long double L2E=1.442695040888963407359924681001892137L; // RN64 by compiler (checked below)
static double DMAX=1.5;
static int pos(double a,double*delta,long double*fo,int*no){
  long double t=L2E*(long double)a; long double n=floorl(t), f=t-n;
  __float128 v=exp2q((__float128)f);           // in [1,2)
  __float128 s=v*0x1p52Q; __float128 fr=s-floorq(s); double d=(double)((fr-0.5Q)*2048.0Q);
  *delta=d; *fo=f; *no=(int)n; return fabs(d)<DMAX;
}
static double nextd(double x,int dir){ return nextafter(x, dir>0?INFINITY:-INFINITY); }
int main(int argc,char**argv){
  if(!strcmp(argv[1],"cluster")){
    double f0=atof(argv[2]); int n=atoi(argv[3]); int cnt=atoi(argv[4]);
    double a=(double)(((long double)n+f0)/L2E); int got=0; long steps=0;
    while(got<cnt && steps<5000000){ double d; long double f; int nn;
      if(pos(a,&d,&f,&nn) && fabs(a)<708.0 ){ printf("%a %+.4f %.12Lf %d\n",a,d,f,nn); got++; }
      a=nextd(a,+1); steps++; }
  } else {
    srand48(atol(argv[2])); int cnt=atoi(argv[3]); int got=0;
    while(got<cnt){ double m=exp2(-4+drand48()*(log2(708.0)+4)); double a=drand48()<0.3? m : -m;
      double d; long double f; int nn; if(pos(a,&d,&f,&nn)){ printf("%a %+.4f %.12Lf %d\n",a,d,f,nn); got++; } }
  }
}
