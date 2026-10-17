/* T-1778: cover count for candidate coarse values hi = A + m*ulp(A) on one run, lo free (one change allowed).
 * Usage: covm RUNFILE RUN MODE m...   (MODE 1: x87 double-rounded coarse product). Build: cc -O2 covm.c -lm */
/* cover count for given hi under rounding-model variants; prints per run best over hi window */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
typedef long double LD;
typedef struct{LD v;int d;}ev;
static int cmp(const void*a,const void*b){const ev*p=a,*q=b;if(p->v<q->v)return -1;if(p->v>q->v)return 1;return q->d-p->d;}
static int cmpx(const void*a,const void*b){double p=*(const double*)a,q=*(const double*)b;return p<q?-1:p>q;}
LD LO[5000],HI[5000];
int cov(int a,int b){static ev evs[10000];int k=0;for(int i=a;i<b;i++){if(LO[i]>HI[i])continue;evs[k].v=LO[i];evs[k++].d=1;evs[k].v=HI[i];evs[k++].d=-1;}
 qsort(evs,k,sizeof(ev),cmp);int cur=0,best=0;for(int i=0;i<k;i++){cur+=evs[i].d;if(cur>best)best=cur;}return best;}
int main(int argc,char**argv){FILE*f=fopen(argv[1],"r");char a[64],b[64];int id; int r0=atoi(argv[2]); int mode=atoi(argv[3]);
 static double P[5000][2]; int n=0;
 while(fscanf(f,"%d %63s %63s",&id,a,b)==3){if(id!=r0)continue;P[n][0]=strtod(a,0);P[n][1]=strtod(b,0);n++;}
 qsort(P,n,sizeof(P[0]),cmpx);
 double A=1.1170334248975802,U=nextafter(A,2)-A;
 for(int c=4;c<argc;c++){long long m=atoll(argv[c]); double hi=A+(double)m*U;
   for(int i=0;i<n;i++){double x=P[i][0],e=P[i][1]; double co;
     if(mode&1){ volatile LD pe=(LD)x*hi; co=(double)pe; } else { volatile double t=x*hi; co=t; }
     LD el=((LD)e+nextafter(e,-1e9))/2, eh=((LD)e+nextafter(e,1e9))/2;
     if(mode&4){ /* x87 final: ext sum rounded to 64-bit then 53: shrink interval by 2^-11 ulp at tie ends if tie->even mismatch (approx: exclude band) */
        LD u=(LD)(nextafter(e,1e9)-e); LD bw=u*ldexpl(1,-12); 
        /* if e mantissa odd, ties round away from e -> exclude bands */
        unsigned long long bits; double ee=e; __builtin_memcpy(&bits,&ee,8); if(bits&1){el+=bw; eh-=bw;} else {el-=bw; eh+=bw;} }
     LD ql=el-co, qh=eh-co;
     if(mode&2){ /* q rounded to double: q in [ql,qh] must be reachable: lo range = preimage of doubles inside [ql,qh] */
        double qa=(double)ql; if((LD)qa<ql) qa=nextafter(qa,1e9); double qb=(double)qh; if((LD)qb>qh) qb=nextafter(qb,-1e9);
        if(qa>qb){LO[i]=1;HI[i]=0;continue;}
        LD ua=((LD)qa-(LD)nextafter(qa,-1e9))/2, ub=((LD)nextafter(qb,1e9)-(LD)qb)/2; ql=(LD)qa-ua; qh=(LD)qb+ub; }
     LO[i]=ql/x; HI[i]=qh/x;}
   int best=cov(0,n); int step=n>64?4:1;
   for(int s=step;s<n;s+=step){int v=cov(0,s)+cov(s,n); if(v>best)best=v;}
   printf("%lld:%d ",m,best);}
 printf("\n");}
