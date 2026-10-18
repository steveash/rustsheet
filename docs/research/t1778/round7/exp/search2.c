// Probe-point searcher (exact-t version). Output: a_hex delta_exact(G) f_floor64 n_floor pred_exactRN64 pred_floorRN64f
// delta = position of exact 2^t64 relative to nearest 53-bit midpoint, units G = ulp53/2048.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <quadmath.h>
static const long double L2E=1.442695040888963407359924681001892137L;
static double DMAX=1.5;
static double rn53rn64(__float128 v){ long double w=(long double)v; return (double)w; } // both RN
static int eval(double a,double*delta,long double*ff,int*no,double*pe,double*pf){
  long double t=L2E*(long double)a; __float128 tq=(__float128)t; __float128 nq=floorq(tq), fq=tq-nq;
  __float128 v=exp2q(fq); __float128 s=v*0x1p52Q; __float128 fr=s-floorq(s); *delta=(double)((fr-0.5Q)*2048.0Q);
  long double n=floorl(t), f=t-n; *ff=f; *no=(int)n;           // x87 floor split: f rounded to 64 bits
  *pe=ldexp(rn53rn64(v),(int)nq); *pf=ldexp(rn53rn64(exp2q((__float128)f)),(int)n);
  return 1;
}
int main(int argc,char**argv){
  double d,pe,pf; long double f; int nn;
  if(!strcmp(argv[1],"cluster")){
    double f0=atof(argv[2]); int n=atoi(argv[3]); int cnt=atoi(argv[4]);
    double a=(double)(((long double)n+f0)/L2E); int got=0; long steps=0;
    while(got<cnt && steps<20000000){ eval(a,&d,&f,&nn,&pe,&pf);
      if(fabs(d)<DMAX && a>-708.0 && a<709.0){ printf("%a %+.5f %.15Lf %d %a %a\n",a,d,f,nn,pe,pf); got++; }
      a=nextafter(a,INFINITY); steps++; }
  } else if(!strcmp(argv[1],"rand")){
    srand48(atol(argv[2])); int cnt=atoi(argv[3]); int got=0;
    while(got<cnt){ double m=exp2(-4+drand48()*(log2(708.0)+4)); double a=drand48()<0.3? m : -m;
      eval(a,&d,&f,&nn,&pe,&pf); if(fabs(d)<DMAX){ printf("%a %+.5f %.15Lf %d %a %a\n",a,d,f,nn,pe,pf); got++; } }
  } else if(!strcmp(argv[1],"eval")){ char ah[64],eh[64],tg[64]; long ce=0,cf=0,dis=0,de=0,df=0,n=0;
    while(scanf("%63s %63s %63s",ah,eh,tg)==3){ double a=strtod(ah,0),E=strtod(eh,0); if(E<0x1p-1022) continue; n++;
      eval(a,&d,&f,&nn,&pe,&pf); ce+=(pe==E); cf+=(pf==E); if(pe!=pf){dis++; de+=(pe==E); df+=(pf==E); printf("DISC %a E=%a exact=%a floor64=%a\n",a,E,pe,pf);} }
    printf("n=%ld exact-t RN53(RN64(2^t)) %ld ; floor-rounded-f %ld ; discriminating %ld: exact wins %ld floor wins %ld\n",n,ce,cf,dis,de,df);
  } else { // splitdisc: a in (-0.693,0) where floor-rounded f changes the prediction
    srand48(atol(argv[2])); int cnt=atoi(argv[3]); int got=0;
    while(got<cnt){ double a=-0.6931*drand48(); if(a==0) continue; eval(a,&d,&f,&nn,&pe,&pf);
      if(pe!=pf){ printf("%a %+.5f %.15Lf %d %a %a\n",a,d,f,nn,pe,pf); got++; } }
  }
}
