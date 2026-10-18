#include "lvl.h"
/* exact random search for poly k (0=P,1=Q) coefficients c2..cn in a box (units: d_j in u2/0.25^(j-2) like apply), other poly fixed.
   usage: qsearch COEF k nsamp seed  lo2 hi2  lo3 hi3 ... (box per d_j, d2 integer)  OUT */
typedef struct { double x,z; int64_t lo,hi; } S;
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int k=atoi(argv[2]); long ns=atol(argv[3]); srand48(atol(argv[4]));
 int n=k?nq:np; double*c=k?Qd:Pd; int nd=n-1; double blo[8],bhi[8]; for(int j=0;j<nd;j++){blo[j]=atof(argv[5+2*j]); bhi[j]=atof(argv[6+2*j]);} const char*out=argv[5+2*nd];
 S*s=malloc(NP*sizeof(S)); long m=0, fixedmiss=0;
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw);
  double dd=k?dist(pw,W):dist(pn,N); double V=k?W:N; double Vlo=1e300,Vhi=-1e300; double v=V; for(int j=0;j<4;j++) v=nextafter(v,-9);
  for(int j=0;j<9;j++){ double rr=k?fin(x,N,v):fin(x,v,W); if(rr==y){ if(v<Vlo)Vlo=v; if(v>Vhi)Vhi=v;} v=nextafter(v,9);} 
  if(Vlo>Vhi){fixedmiss++; continue;}
  if(fabs(dd)<0.2 || V<Vlo || V>Vhi){ s[m].x=x; s[m].z=z; s[m].lo=B(Vlo); s[m].hi=B(Vhi); m++; } }
 fprintf(stderr,"sensitive %ld unfixable %ld\n",m,fixedmiss);
 double base[8]; memcpy(base,c,sizeof base); double u2=nextafter(c[2],9)-c[2]; long best=-1; double bestc[8];
 for(long it=0;it<ns;it++){ for(int j=0;j<nd;j++){ double d=blo[j]+(bhi[j]-blo[j])*drand48(); if(j==0) d=nearbyint(d); if(it==0) d=(j==0)?nearbyint(0.5*(blo[0]+bhi[0])):0.5*(blo[j]+bhi[j]); c[2+j]=(double)((LD)base[2+j]+(LD)d*(LD)u2/powl(0.25L,j)); }
  long sc=0; for(long i=0;i<m;i++){ double V=horner(n,c,s[i].z,0); int64_t b=B(V); sc+= (b>=s[i].lo && b<=s[i].hi); }
  if(sc>best){ best=sc; memcpy(bestc,c,sizeof bestc); fprintf(stderr,"it %ld sens-score %ld/%ld\n",it,sc,m); if(sc==m) break; } }
 memcpy(c,bestc,sizeof bestc); savecoef(out); printf("best %ld/%ld (unfixable-with-other-fixed %ld)\n",best,m,fixedmiss); }
