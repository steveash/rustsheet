#include "common.h"
#include <stdint.h>
static int64_t B(double d){int64_t i; memcpy(&i,&d,8); return i;} static double D(int64_t i){double d; memcpy(&d,&i,8); return d;}
static double dist(LD pre,double V){ double up=nextafter(V,9),dn=nextafter(V,-9); LD a=(pre-((LD)V+dn)/2)/(up-V), b=(pre-((LD)V+up)/2)/(up-V); return (double)(fabsl(a)<fabsl(b)?a:b);}
static inline double addc(double t,double c){ volatile LD u=(LD)t+(LD)c; return st(u);} static inline double mulz(double s,double z){ volatile LD u=(LD)s*(LD)z; return st(u);} 
/* given interval [vlo,vhi] (bits) of output of addc(t,c), find t interval (bits) near guess g */
static int inv_add(int64_t vlo,int64_t vhi,double c,int64_t g,int64_t*olo,int64_t*ohi){ int64_t a=g-(1LL<<44),b=g+(1LL<<44),lo,hi; if(a<0)a=0;
 lo=a;hi=b; while(lo<hi){int64_t m=lo+(hi-lo)/2; if(B(addc(D(m),c))>=vlo) hi=m; else lo=m+1;} *olo=lo;
 lo=a;hi=b; while(lo<hi){int64_t m=lo+(hi-lo+1)/2; if(B(addc(D(m),c))<=vhi) lo=m; else hi=m-1;} *ohi=lo; return *olo<=*ohi; }
static int inv_mul(int64_t vlo,int64_t vhi,double z,int64_t g,int64_t*olo,int64_t*ohi){ int64_t a=g-(1LL<<44),b=g+(1LL<<44),lo,hi; if(a<0)a=0;
 lo=a;hi=b; while(lo<hi){int64_t m=lo+(hi-lo)/2; if(B(mulz(D(m),z))>=vlo) hi=m; else lo=m+1;} *olo=lo;
 lo=a;hi=b; while(lo<hi){int64_t m=lo+(hi-lo+1)/2; if(B(mulz(D(m),z))<=vhi) lo=m; else hi=m-1;} *ohi=lo; return *olo<=*ohi; }
/* LIS-like max feasible non-decreasing selection; points already sorted by z. intervals lo[i],hi[i] */
static int cmp64(const void*a,const void*b){int64_t x=*(int64_t*)a,y=*(int64_t*)b; return x<y?-1:x>y;}
static long maxmono(long n,const int64_t*lo,const int64_t*hi,char*keep){
 /* multiset via sorted coordinate + fenwick */
 int64_t*v=malloc(n*8); for(long i=0;i<n;i++) v[i]=lo[i]; qsort(v,n,8,cmp64); long m=n; int*fw=calloc(m+1,4); long size=0;
 for(long i=0;i<n;i++){ /* insert lo */ long p=0,L=0,R=m; while(L<R){long M=(L+R)/2; if(v[M]<lo[i]) L=M+1; else R=M;} p=L; for(long j=p+1;j<=m;j+=j&-j) fw[j]++; size++;
   /* remove smallest element > hi: first index q with v[q]>hi; find first present element with idx>=q */
   L=0;R=m; while(L<R){long M=(L+R)/2; if(v[M]<=hi[i]) L=M+1; else R=M;} long q=L; /* count of present with idx<q */
   long c=0; for(long j=q;j>0;j-=j&-j) c+=fw[j]; if(c<size){ /* find (c+1)-th element */ long pos=0,rem=c+1; for(int b=20;b>=0;b--){ long np2=pos+(1L<<b); if(np2<=m && fw[np2]<rem){pos=np2; rem-=fw[np2];} } /* pos is index (0-based) */ for(long j=pos+1;j<=m;j+=j&-j) fw[j]--; size--; if(keep) keep[i]=0; } }
 free(v); free(fw); return size; }
