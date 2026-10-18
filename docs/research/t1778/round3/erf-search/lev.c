#include "common.h"
/* horner with per-step tracing; perturb step j (0..2n-1: even=product t, odd=sum s) by d ulps */
static double hp(int n,const double*c,double z,int pj,int d,double*fr){ double s=c[n]; int j=0;
 for(int i=n-1;i>=0;i--){ volatile LD t=(LD)s*(LD)z; double tt=st(t); if(fr){double u=nextafter(tt,9)-tt; fr[j]=(double)((t-(LD)tt)/u);} if(j==pj) for(int k=0;k<abs(d);k++) tt=nextafter(tt,d>0?9:-9); j++;
  volatile LD uu=(LD)tt+(LD)c[i]; s=st(uu); if(fr){double u=nextafter(s,9)-s; fr[j]=(double)((uu-(LD)s)/u);} if(j==pj) for(int k=0;k<abs(d);k++) s=nextafter(s,d>0?9:-9); j++; }
 return s; }
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]);
 long fix[2][12]={{0}}, fixp[2][12]={{0}}; long miss=0; double fr[12];
 FILE*o=fopen("lev_miss.txt","w");
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); double N=hp(np,Pd,z,-1,0,0),W=hp(nq,Qd,z,-1,0,0);
  if(fin(x,N,W)==y) continue; miss++; fprintf(o,"%d %.5e",P_[i].f,z);
  for(int k=0;k<2;k++){ int n=k?nq:np; const double*c=k?Qd:Pd; hp(n,c,z,-1,0,fr);
   for(int j=0;j<2*n;j++){ double V=hp(n,c,z,j,-1,0); double N2=k?N:V, W2=k?V:W; int f1=fin(x,N2,W2)==y; fix[k][j]+=f1;
     V=hp(n,c,z,j,+1,0); N2=k?N:V; W2=k?V:W; int f2=fin(x,N2,W2)==y; fixp[k][j]+=f2; fprintf(o," %c%d:%+.3f%s",k?'Q':'P',j,fr[j],f1?"F":(f2?"f":"")); } }
  fprintf(o,"\n"); }
 printf("misses %ld\n",miss); for(int k=0;k<2;k++){ printf("%c -1ulp fixes per step (t%d,s%d..):",k?'Q':'P',0,0); for(int j=0;j<2*(k?nq:np);j++) printf(" %ld",fix[k][j]); printf("\n   +1:"); for(int j=0;j<2*(k?nq:np);j++) printf(" %ld",fixp[k][j]); printf("\n"); } }
