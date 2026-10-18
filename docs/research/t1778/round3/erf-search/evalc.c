#include "lvl.h"
/* evalc COEF k SAMPLES OUT [thr]: each line "d2 d3 ... dn" applied to poly k (as in apply, c1 unchanged); exact score on sensitive pts; write best and full score */
typedef struct { double x,z; int64_t lo,hi; } S;
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int k=atoi(argv[2]); double thr=argc>5?atof(argv[5]):0.08;
 int n=k?nq:np; double*c=k?Qd:Pd; int nd=n-1; S*s=malloc(NP*sizeof(S)); long m=0;
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw);
  double dd=k?dist(pw,W):dist(pn,N); double V=k?W:N; double Vlo=1e300,Vhi=-1e300; double v=V; for(int j=0;j<4;j++) v=nextafter(v,-9);
  for(int j=0;j<9;j++){ double rr=k?fin(x,N,v):fin(x,v,W); if(rr==y){ if(v<Vlo)Vlo=v; if(v>Vhi)Vhi=v;} v=nextafter(v,9);} 
  if(Vlo>Vhi){ Vlo=1; Vhi=0; }
  if(fabs(dd)<thr || V<Vlo || V>Vhi){ s[m].x=x; s[m].z=z; s[m].lo=B(Vlo); s[m].hi=B(Vhi); m++; } }
 fprintf(stderr,"sensitive %ld\n",m); double base[8]; memcpy(base,c,sizeof base); double u2=nextafter(c[2],9)-c[2];
 FILE*f=fopen(argv[3],"r"); double d[8]; long best=-1,cnt=0,zero=0; double bestc[8];
 while(1){ int ok=1; for(int j=0;j<nd;j++) if(fscanf(f,"%lf",&d[j])!=1){ok=0;break;} if(!ok)break; cnt++;
  for(int j=0;j<nd;j++){ double dj=(j==0)?nearbyint(d[j]):d[j]; c[2+j]=(double)((LD)base[2+j]+(LD)dj*(LD)u2/powl(0.25L,j)); }
  long sc=0; for(long i=0;i<m;i++){ int64_t b=B(horner(n,c,s[i].z,0)); sc+=(b>=s[i].lo&&b<=s[i].hi);} if(sc==m) zero++;
  if(sc>best){best=sc; memcpy(bestc,c,sizeof bestc);} }
 memcpy(c,bestc,sizeof bestc); savecoef(argv[4]);
 long h=0; for(long i=0;i<NP;i++){ double x=P_[i].x; double z=st((LD)x*x); h+=fin(x,horner(np,Pd,z,0),horner(nq,Qd,z,0))==P_[i].y; }
 printf("samples %ld, all-sensitive-ok %ld, best sens %ld/%ld, full score of best %ld/%ld\n",cnt,zero,best,m,h,NP); }
