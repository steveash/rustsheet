#include "lvl.h"
/* level-2 test for poly k: scan c1 offsets; report max monotone-feasible count for s2 */
typedef struct { double z; int64_t L1,U1; double s1,s2,t4; } R;
static int cz(const void*a,const void*b){ double x=((R*)a)->z,y=((R*)b)->z; return x<y?-1:x>y; }
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int k=atoi(argv[2]); double zmin=atof(argv[3]); int lo_off=atoi(argv[4]),hi_off=atoi(argv[5]),step=atoi(argv[6]);
 int n=k?nq:np; double*c=k?Qd:Pd; R*r=malloc(NP*sizeof(R)); long m=0;
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); if(z<zmin) continue; LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw);
  if(fabs(k?dist(pn,N):dist(pw,W))<0.2) continue;
  double V=k?W:N; double Vlo=1e300,Vhi=-1e300; double v=V; for(int j=0;j<5;j++) v=nextafter(v,-9);
  for(int j=0;j<11;j++){ double rr=k?fin(x,N,v):fin(x,v,W); if(rr==y){ if(v<Vlo)Vlo=v; if(v>Vhi)Vhi=v;} v=nextafter(v,9);} if(Vlo>Vhi) continue;
  double s=c[n],t=0,s2=0,t4=0; for(int j=n-1;j>=0;j--){ t=mulz(s,z); if(j==0) break; if(j==1){s2=s;t4=t;} s=addc(t,c[j]); }
  int64_t tlo,thi,L1,U1; inv_add(B(Vlo),B(Vhi),c[0],B(t),&tlo,&thi); inv_mul(tlo,thi,z,B(s),&L1,&U1);
  if(L1>B(s)-64*8 && U1<B(s)+64*8 && 0){} 
  r[m].z=z; r[m].L1=L1; r[m].U1=U1; r[m].s1=s; r[m].s2=s2; r[m].t4=t4; m++; }
 qsort(r,m,sizeof(R),cz); fprintf(stderr,"points %ld\n",m);
 int64_t*lo=malloc(m*8),*hi=malloc(m*8);
 for(int off=lo_off;off<=hi_off;off+=step){ double c1=D(B(c[1])+off); long bad=0;
  for(long i=0;i<m;i++){ int64_t a,b; if(!inv_add(r[i].L1,r[i].U1,c1,B(r[i].t4),&a,&b)){ lo[i]=1;hi[i]=0; bad++; continue;} int64_t e,f; if(!inv_mul(a,b,r[i].z,B(r[i].s2),&e,&f)){bad++; e=1;f=0;} lo[i]=e; hi[i]=f; }
  /* empty intervals: set lo>hi -> maxmono will drop them? insert lo, remove first>hi -> removes itself or other. */
  long best=maxmono(m,lo,hi,0); printf("c1off %d infeasible-empty %ld maxfeasible %ld of %ld (excluded %ld)\n",off,bad,best,m,m-best); fflush(stdout); }
}
