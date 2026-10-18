#include "common.h"
/* store pattern: bit j set => op j is stored (rounded fl53(fl64)); else kept in 80-bit register. last op always stored */
static double hpat(int n,const double*c,double z,unsigned pat){ LD s=c[n]; int j=0;
 for(int i=n-1;i>=0;i--){ volatile LD t=s*(LD)z; LD tt = (pat>>j&1)? (LD)st(t): t; j++; volatile LD uu=tt+(LD)c[i]; s=(pat>>j&1)?(LD)st(uu):uu; j++; }
 return st(s); }
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int which=atoi(argv[2]);
 /* subsample: only points that are 'sensitive' for speed? use all */
 int n= which? nq:np; unsigned full=(1u<<(2*n))-1;
 double *Nfix=malloc(NP*8),*Wfix=malloc(NP*8),*zz=malloc(NP*8);
 for(long i=0;i<NP;i++){ zz[i]=st((LD)P_[i].x*P_[i].x); Nfix[i]=hpat(np,Pd,zz[i],(1u<<(2*np))-1); Wfix[i]=hpat(nq,Qd,zz[i],(1u<<(2*nq))-1);} 
 for(unsigned pat=0;pat<=full;pat++){ if(!(pat>>(2*n-1)&1)) continue; long h=0;
  for(long i=0;i<NP;i++){ double x=P_[i].x; double N=Nfix[i],W=Wfix[i]; if(which) W=hpat(nq,Qd,zz[i],pat); else N=hpat(np,Pd,zz[i],pat); h+=fin(x,N,W)==P_[i].y; }
  printf("%c pat %03x hit %ld\n",which?'Q':'P',pat,h); fflush(stdout);} }
