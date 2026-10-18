#include "common.h"
/* per point near a midpoint (either poly within 0.08 ulp above mid) : output z-error, dists */
static double dabove(LD pre,double V){ double up=nextafter(V,9),dn=nextafter(V,-9); LD midD=((LD)V+dn)/2, midU=((LD)V+up)/2; LD u=up-V;
  LD a=(pre-midD)/u, b=(pre-midU)/u; return (double)(fabsl(a)<fabsl(b)?a:b); }
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]);
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); double ze=fma(x,x,-z); double uz=nextafter(z,9)-z;
  LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw); int hit=fin(x,N,W)==y;
  double aN=dabove(pn,N), aW=dabove(pw,W); if(fabs(aN)>0.1&&fabs(aW)>0.1) continue;
  int nN=0,nW=0; if(!hit){nN=fin(x,nextafter(N,-9),W)==y; nW=fin(x,N,nextafter(W,-9))==y; if(!nN&&!nW){nN=fin(x,nextafter(N,9),W)==y?2:0; nW=fin(x,N,nextafter(W,9))==y?2:0;}}
  /* derivative-based shift for -1 ulp z, in units of ulp(N)/ulp(W) */
  double dz=uz; double Np=horner(np,Pd,z+dz*1e6,0), Wp=horner(nq,Qd,z+dz*1e6,0);
  double sN=(Np-N)/1e6/(nextafter(N,9)-N), sW=(Wp-W)/1e6/(nextafter(W,9)-W);
  printf("%d %d %.6e %+.4f %+.5f %+.5f %d %d %.5f %.5f\n",P_[i].f,hit,z,ze/uz,aN,aW,nN,nW,sN,sW); } }
