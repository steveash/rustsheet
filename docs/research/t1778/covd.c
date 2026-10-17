/* T-1778: cover over a constant lo offset with lo = erf(x)/x + d0 - H(z) + K*(fl(W) - W), W = sqrt(1-2*BETA*z)
 * (WF=1) or exp(-BETA*z) (WF=0). KW=0 drops the W term so the printed offset carries the W sawtooth.
 * Usage: BETA=b covd RUNFILE RUN NH h0 h1 .. m...   Build: cc -O2 covd.c -lquadmath -lm */
/* cover over constant delta: lo_i = S(z)+d0 - H(z) + K*(Wd - e^{-bz}) + delta ; hi candidates given */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <quadmath.h>
typedef long double LD; typedef __float128 Q;
typedef struct{LD v;int d;}ev;
static int cmp(const void*a,const void*b){const ev*p=a,*q=b;if(p->v<q->v)return -1;if(p->v>q->v)return 1;return q->d-p->d;}
int main(int argc,char**argv){FILE*f=fopen(argv[1],"r");char a[64],b[64];int id; int r0=atoi(argv[2]);
 Q h[8]={0}; int nh=atoi(argv[3]); for(int k=0;k<nh;k++) h[k]=strtoflt128(argv[4+k],0);
 Q K=getenv("KW")?strtoflt128(getenv("KW"),0):1.1366973296084131Q, beta=strtoflt128(getenv("BETA"),0), d0=1.07e-16Q; int WF=getenv("WF")?atoi(getenv("WF")):1;
 static double X[5000],E[5000]; int n=0;
 while(fscanf(f,"%d %63s %63s",&id,a,b)==3){if(id!=r0)continue;X[n]=strtod(a,0);E[n]=strtod(b,0);n++;}
 double A=1.1170334248975802,U=nextafter(A,2)-A;
 static LD L0[5000];
 for(int i=0;i<n;i++){double x=X[i]; double zd=x*x; Q z=zd; Q H=A; Q zp=1; for(int k=1;k<nh;k++){zp*=z; H+=h[k]*zp;}
   Q S=erfq((Q)x)/(Q)x+d0; Q We = WF? sqrtq(1-2*beta*z) : expq(-beta*z); double Wd=(double)We; Q lo=S-H+K*((Q)Wd-We); L0[i]=(LD)lo;}
 static ev evs[10000];
 for(int c=4+nh;c<argc;c++){long long m=atoll(argv[c]); double hi=A+(double)m*U; int k=0;
   for(int i=0;i<n;i++){double x=X[i],e=X[i]?E[i]:0; volatile LD pe=(LD)x*hi; double co=(double)pe;
     LD el=((LD)e+nextafter(e,-1e9))/2, eh=((LD)e+nextafter(e,1e9))/2;
     evs[k].v=(el-co)/x-L0[i]; evs[k++].d=1; evs[k].v=(eh-co)/x-L0[i]; evs[k++].d=-1;}
   qsort(evs,k,sizeof(ev),cmp); int cur=0,best=0; LD bl=0,bh=0; for(int i=0;i<k;i++){cur+=evs[i].d; if(cur>best){best=cur;bl=evs[i].v;bh=evs[i+1].v;}}
   printf("%lld %d %.6Le %.6Le\n",m,best,bl,bh);}
 printf("\n");}
