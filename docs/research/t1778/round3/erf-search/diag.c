#include "common.h"
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]);
 FILE*o=fopen(argv[2],"w"); long hit=0;
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw);
  double fN=(double)((pn-(LD)N)/(LD)(nextafter(N,9)-N)), fW=(double)((pw-(LD)W)/(LD)(nextafter(W,9)-W));
  int dn=99,dw=99; if(fin(x,N,W)==y){dn=dw=0;hit++;} else { int best=99; for(int a=-2;a<=2;a++)for(int b=-2;b<=2;b++){ double N2=N,W2=W; for(int k=0;k<abs(a);k++)N2=nextafter(N2,a>0?9:-9); for(int k=0;k<abs(b);k++)W2=nextafter(W2,b>0?9:-9); if(fin(x,N2,W2)==y && abs(a)+abs(b)<best){best=abs(a)+abs(b);dn=a;dw=b;} } }
  fprintf(o,"%d %d %a %.6e %+.5f %+.5f %d %d\n",P_[i].f,P_[i].r,x,z,fN,fW,dn,dw);}
 fprintf(stderr,"hit %ld/%ld\n",hit,NP); }
