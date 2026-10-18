#include "common.h"
#include <fenv.h>
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int zm=atoi(argv[2]);
 long hit=0, nf[16]={0}, hf[16]={0}, neg=0;
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; if(x<0)neg++; double z;
  LD e=(LD)x*(LD)x; /* exact in 64 bits? x has 53 bits -> product 106 bits, not exact */
  if(zm==0) z=st(e);
  else { fesetround(zm==1?FE_DOWNWARD:FE_TOWARDZERO); volatile double xx=x; volatile double t=xx*xx; z=t; fesetround(FE_TONEAREST);} 
  double N=horner(np,Pd,z,0),W=horner(nq,Qd,z,0); int h=fin(x,N,W)==y; hit+=h; nf[P_[i].f]++; hf[P_[i].f]+=h; }
 for(int f=0;f<12;f++) printf("f%d %ld/%ld\n",f,hf[f],nf[f]); printf("hit %ld/%ld neg=%ld\n",hit,NP,neg); }
