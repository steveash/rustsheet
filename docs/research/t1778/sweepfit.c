/* T-1778 (build: cc -O2 sweepfit.c -lquadmath -lm). Sweep-level joint fit: one hi alias m (absolute, hi_i = fl(H(z_i)) with H shifted by m ulps), one V phase u0 at z_ref,
 * one common lo offset (cover). args: FILE RUN_FIRST RUN_LAST m_center WINDOW NU slope zref */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <quadmath.h>
typedef long double LD; typedef __float128 Q;
typedef struct{LD v;int d;}ev;
static int cmp(const void*a,const void*b){const ev*p=a,*q=b;if(p->v<q->v)return -1;if(p->v>q->v)return 1;return q->d-p->d;}
#define NMAX 8000
static double X[NMAX],CO[NMAX]; static LD L0[NMAX],DZ[NMAX],EL[NMAX],EH[NMAX],HU[NMAX]; static int n; static ev evs[2*NMAX];
int main(int argc,char**argv){FILE*f=fopen(argv[1],"r");char s1[64],s2[64];int id; int ra=atoi(argv[2]),rb=atoi(argv[3]); long long mc=atoll(argv[4]); int W=atoi(argv[5]); int NU=atoi(argv[6]); int NF=getenv("NF")?atoi(getenv("NF")):1; LD s=strtold(argv[7],0); LD zref=strtold(argv[8],0);
 Q h[5]={0,0.14068131264230Q,0.0448191707Q,0.00180525Q,0.00018Q}; Q K=1.1366973296084131Q, d0=1.07e-16Q;
 double A=1.1170334248975802,U=nextafter(A,2)-A; LD G=ldexpl(1,-53);
 while(fscanf(f,"%d %63s %63s",&id,s1,s2)==3){ if(id<ra||id>rb) continue; double x=strtod(s1,0); double E=strtod(s2,0); X[n]=x; double zd=x*x; Q z=zd;
   Q H=A, zp=1; for(int j=1;j<5;j++){zp*=z; H+=h[j]*zp;}
   HU[n]=(LD)((H-(Q)A)/(Q)U); L0[n]=(LD)(erfq((Q)x)/(Q)x+d0-H); DZ[n]=(LD)zd-zref;
   EL[n]=((LD)E+nextafter(E,-1e9))/2; EH[n]=((LD)E+nextafter(E,1e9))/2; n++; }
 int best=-1; static int tab[401][2000]; int bj=0,bf=0,bt=0; static int tabf[64];
 for(int fi=0;fi<NF;fi++) for(int j=0;j<2*W+1;j++){ long long dm=mc-W+j; LD fr=(LD)fi/NF; /* hi = A + round(HU + frac)*U with integer shift dm relative to model */
   for(int i=0;i<n;i++){ LD hu=HU[i]+(LD)(dm-mc)+fr; double hi=A+(double)llroundl(hu)*U; volatile LD pe=(LD)X[i]*hi; CO[i]=(double)pe; }
   for(int t=0;t<NU;t++){ LD u0=(LD)t/NU; int k=0;
     for(int i=0;i<n;i++){ LD u=u0+s*DZ[i]; LD lo=L0[i]+(LD)K*G*(u-roundl(u));
       evs[k].v=(EL[i]-CO[i])/X[i]-lo; evs[k++].d=1; evs[k].v=(EH[i]-CO[i])/X[i]-lo; evs[k++].d=-1;}
     qsort(evs,k,sizeof(ev),cmp); int cur=0,bb=0; for(int i=0;i<k;i++){cur+=evs[i].d; if(cur>bb)bb=cur;}
     if(bb>tabf[fi])tabf[fi]=bb; if(fi==0||bb>tab[j][t]) tab[j][t]=bb; if(bb>best){best=bb;bj=j;bf=fi;bt=t;} } }
 printf("runs %d-%d n %d best %d :",ra,rb,n,best);
 for(int j=0;j<2*W+1;j++){ for(int t=0;t<NU;){ if(tab[j][t]>=best-0){int a=t; while(t<NU&&tab[j][t]>=best)t++; printf(" m%+d[%.4f,%.4f]",j-W,(double)a/NU,(double)(t-1)/NU);} else t++; } }
 int second=-1; for(int j=0;j<2*W+1;j++) for(int t=0;t<NU;t++) if(tab[j][t]<best && tab[j][t]>second) second=tab[j][t];
 printf("  second %d  bestfrac %.3f m%+d u %.4f |",second,(double)bf/NF,bj-W,(double)bt/NU); for(int fi=0;fi<NF;fi++) printf(" %d",tabf[fi]); printf("\n");
}
