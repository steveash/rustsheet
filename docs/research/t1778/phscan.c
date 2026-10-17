/* T-1778: per run, scan the coarse alias m and the W rounding phase u0 jointly (lo = erf/x + d0 - H + K*G*(u - round(u))).
 * Usage: phscan RUNFILE RUN m_center WINDOW NPHASE BETA   Build: cc -O2 phscan.c -lquadmath -lm */
/* per run: scan hi alias m (window) and W phase u0 in [0,1); lo = S+d0-H + K*G*(u-round(u)), u = u0 + s*(z-z0) */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <quadmath.h>
typedef long double LD; typedef __float128 Q;
typedef struct{LD v;int d;}ev;
static int cmp(const void*a,const void*b){const ev*p=a,*q=b;if(p->v<q->v)return -1;if(p->v>q->v)return 1;return q->d-p->d;}
int main(int argc,char**argv){FILE*f=fopen(argv[1],"r");char s1[64],s2[64];int id; int r0=atoi(argv[2]); long long mc=atoll(argv[3]); int W=atoi(argv[4]); int NU=atoi(argv[5]); double b=atof(argv[6]);
 Q h[5]={0,0.14068131264230Q,0.0448191707Q,0.00180525Q,0.00018Q}; Q K=1.1366973296084131Q, d0=1.07e-16Q;
 double A=1.1170334248975802,U=nextafter(A,2)-A; LD G=ldexpl(1,-53);
 static double X[1100],E[1100]; static LD L0[1100],DZ[1100]; int n=0; double z0=0;
 while(fscanf(f,"%d %63s %63s",&id,s1,s2)==3){ if(id!=r0) continue; double x=strtod(s1,0); X[n]=x; E[n]=strtod(s2,0); double zd=x*x; if(n==0) z0=zd; Q z=zd;
   Q H=A; Q zp=1; for(int j=1;j<5;j++){zp*=z; H+=h[j]*zp;}
   L0[n]=(LD)(erfq((Q)x)/(Q)x+d0-H); DZ[n]=(LD)zd-(LD)z0; n++; }
 LD s=(LD)b/sqrtl(1-2*(LD)b*z0)/G;
 static ev evs[2400]; int best=-1; long long bm=0; LD bu=0; static int tab[200][1000];
 for(int j=0;j<2*W+1;j++){ long long m=mc-W+j; double hi=A+(double)m*U; static double CO[1100];
   for(int i=0;i<n;i++){ volatile LD pe=(LD)X[i]*hi; CO[i]=(double)pe; }
   for(int t=0;t<NU;t++){ LD u0=(LD)t/NU; int k=0;
     for(int i=0;i<n;i++){ LD u=u0+s*DZ[i]; LD lo=L0[i]+(LD)K*G*(u-roundl(u)); double e=E[i];
       LD el=((LD)e+nextafter(e,-1e9))/2, eh=((LD)e+nextafter(e,1e9))/2;
       evs[k].v=(el-CO[i])/X[i]-lo; evs[k++].d=1; evs[k].v=(eh-CO[i])/X[i]-lo; evs[k++].d=-1;}
     qsort(evs,k,sizeof(ev),cmp); int cur=0,bb=0; for(int i=0;i<k;i++){cur+=evs[i].d; if(cur>bb)bb=cur;}
     tab[j][t]=bb; if(bb>best){best=bb;bm=m;bu=u0;} } }
 /* report all (m, u0-interval) achieving best */
 printf("%d z0 %.17g n %d best %d :",r0,z0,n,best);
 for(int j=0;j<2*W+1;j++){ for(int t=0;t<NU;){ if(tab[j][t]==best){int a=t; while(t<NU&&tab[j][t]==best)t++; printf(" m%+d[%.3f,%.3f]",j-W,(double)a/NU,(double)(t-1)/NU);} else t++; } }
 printf("\n");
}
