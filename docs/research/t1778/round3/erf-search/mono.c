#include "common.h"
#include <stdint.h>
/* Coefficient-free monotonicity test at the s1 level.
 * For poly k (0=P,1=Q): acceptable final value V interval given the other poly fixed at model value (only if other's pre is >0.2ulp from midpoint).
 * t_last interval: doubles t with fl53(fl64(t + c0)) in [Vlo,Vhi]; s1 interval: doubles s with fl53(fl64(s*z)) in [tlo,thi].
 * Output per point: z s1lo s1hi (as int64 bits) and current model s1. */
static int64_t B(double d){int64_t i; memcpy(&i,&d,8); return i;} static double D(int64_t i){double d; memcpy(&d,&i,8); return d;}
static double dist(LD pre,double V){ double up=nextafter(V,9),dn=nextafter(V,-9); LD a=(pre-((LD)V+dn)/2)/(up-V), b=(pre-((LD)V+up)/2)/(up-V); return (double)(fabsl(a)<fabsl(b)?a:b);}
static double c0g; static double addc(double t){ volatile LD u=(LD)t+(LD)c0g; return st(u);} static double mulz(double s,double z){ volatile LD u=(LD)s*(LD)z; return st(u);} 
/* smallest int64 i in [lo,hi] with f(i)>=target (f monotone) */
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int k=atoi(argv[2]); double zmin=atof(argv[3]);
 int n=k?nq:np; double*c=k?Qd:Pd; c0g=c[0];
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); if(z<zmin) continue; LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw);
  double other_d = k? dist(pn,N): dist(pw,W); if(fabs(other_d)<0.2) continue;
  double V=k?W:N; double Vlo=1e300,Vhi=-1e300; double v=V; for(int j=0;j<5;j++) v=nextafter(v,-9);
  for(int j=0;j<11;j++){ double r=k?fin(x,N,v):fin(x,v,W); if(r==y){ if(v<Vlo)Vlo=v; if(v>Vhi)Vhi=v;} v=nextafter(v,9);} 
  if(Vlo>Vhi){ printf("NOACC %d %a\n",P_[i].f,x); continue;}
  /* compute s1 (model) */
  double s=c[n]; double s1=0,t=0; for(int j=n-1;j>=0;j--){ t=mulz(s,z); if(j==0) break; s=st((LD)t+(LD)c[j]); } s1=s; 
  /* t interval: scan bisection over t bits */
  int64_t tb=B(t), lo,hi,a,b;
  /* tlo = min t with addc(t)>=Vlo */
  a=tb-(1LL<<40); b=tb+(1LL<<40); if(a<0)a=0; lo=a; hi=b; while(lo<hi){int64_t m=lo+(hi-lo)/2; if(addc(D(m))>=Vlo) hi=m; else lo=m+1;} int64_t tlo=lo;
  lo=a; hi=b; while(lo<hi){int64_t m=lo+(hi-lo+1)/2; if(addc(D(m))<=Vhi) lo=m; else hi=m-1;} int64_t thi=lo;
  int64_t sb=B(s1); a=sb-(1LL<<40); b=sb+(1LL<<40);
  lo=a; hi=b; while(lo<hi){int64_t m=lo+(hi-lo)/2; if(B(mulz(D(m),z))>=tlo) hi=m; else lo=m+1;} int64_t slo=lo;
  lo=a; hi=b; while(lo<hi){int64_t m=lo+(hi-lo+1)/2; if(B(mulz(D(m),z))<=thi) lo=m; else hi=m-1;} int64_t shi=lo;
  printf("%a %lld %lld %lld %d %d\n",z,(long long)(slo-sb),(long long)(shi-sb),(long long)sb,P_[i].f,fin(x,N,W)==y);
 } }
