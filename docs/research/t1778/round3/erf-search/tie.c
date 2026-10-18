#include "common.h"
/* classify points by exact position of pre-value relative to midpoint, per polynomial; and t properties */
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]);
 long cnt[2][3][2]={{{0}}}; // poly, class(0 tie-roundedup,1 tie-roundeddown,2 near-not-tie), miss
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw);
  int hit=fin(x,N,W)==y; int needN=0,needW=0;
  if(!hit){ needN = fin(x,nextafter(N,-9),W)==y; needW = fin(x,N,nextafter(W,-9))==y; }
  for(int k=0;k<2;k++){ LD pre=k?pw:pn; double V=k?W:N; double up=nextafter(V,9),dn=nextafter(V,-9);
   LD midU=((LD)V+(LD)up)/2, midD=((LD)V+(LD)dn)/2; int cls=-1;
   if(pre==midD) cls=0; else if(pre==midU) cls=1; else if (fabsl(pre-midD)<0.01L*(up-V)||fabsl(pre-midU)<0.01L*(up-V)) cls=2;
   if(cls>=0){ int m = !hit && (k? needW: needN); cnt[k][cls][m]++; } } }
 const char*nm[3]={"tie,rounded UP (pre=lower mid)","tie,rounded DOWN (pre=upper mid)","within .01 not tie"};
 for(int k=0;k<2;k++)for(int c=0;c<3;c++) printf("%s %-34s n=%ld miss=%ld\n",k?"W":"N",nm[c],cnt[k][c][0]+cnt[k][c][1],cnt[k][c][1]); }
