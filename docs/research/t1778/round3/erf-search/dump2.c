#include "lvl.h"
/* For poly k and c1 offset: dump per point z, and interval for G(z)=c2+c3 z+...(+c_n z^(n-2)) relative to current G0, in units of u2=ulp(s2).
   Only points with tight side (within W ulps) are dumped. Also dumps 'empty' points. */
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int k=atoi(argv[2]); double zmin=atof(argv[3]); int off=atoi(argv[4]); double win=atof(argv[5]);
 int n=k?nq:np; double*c=k?Qd:Pd; double c1=D(B(c[1])+off); long empty=0,m=0;
 for(long i=0;i<NP;i++){ double x=P_[i].x,y=P_[i].y; double z=st((LD)x*x); if(z<zmin) continue; LD pn,pw; double N=horner(np,Pd,z,&pn),W=horner(nq,Qd,z,&pw);
  if(fabs(k?dist(pn,N):dist(pw,W))<0.2) continue;
  double V=k?W:N; double Vlo=1e300,Vhi=-1e300; double v=V; for(int j=0;j<5;j++) v=nextafter(v,-9);
  for(int j=0;j<11;j++){ double rr=k?fin(x,N,v):fin(x,v,W); if(rr==y){ if(v<Vlo)Vlo=v; if(v>Vhi)Vhi=v;} v=nextafter(v,9);} if(Vlo>Vhi) continue;
  double s=c[n],t=0,s2=0,t4=0; for(int j=n-1;j>=0;j--){ t=mulz(s,z); if(j==0) break; if(j==1){s2=s;t4=t;} s=addc(t,c[j]); }
  int64_t tlo,thi,L1,U1,a,b,e,f; inv_add(B(Vlo),B(Vhi),c[0],B(t),&tlo,&thi); inv_mul(tlo,thi,z,B(s),&L1,&U1);
  int ok=inv_add(L1,U1,c1,B(t4),&a,&b); if(ok) ok=inv_mul(a,b,z,B(s2),&e,&f);
  if(!ok){ empty++; printf("E %a %d %a\n",z,P_[i].f,x); continue; }
  LD G0=0; for(int j=n;j>=2;j--) G0=G0*(LD)z+(LD)c[j];
  double u2=nextafter(s2,9)-s2; /* assume same binade */
  double lo=(double)(((LD)D(e)-(LD)u2/2-G0)/u2), hi=(double)(((LD)D(f)+(LD)u2/2-G0)/u2);
  if(lo>-win || hi<win){ printf("C %a %.6f %.6f %d %a\n",z,lo,hi,P_[i].f,x); m++; } }
 fprintf(stderr,"off %d dumped %ld empty %ld\n",off,m,empty); }
