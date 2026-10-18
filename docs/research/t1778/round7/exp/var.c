#include <stdio.h>
#include <stdlib.h>
#include <math.h>
static long double f2xm1(long double f){ long double u; __asm__ volatile("f2xm1":"=t"(u):"0"(f)); return u; }
static long double fscale(long double w,long double n){ long double r; __asm__ volatile("fscale":"=t"(r):"0"(w),"u"(n)); return r; }
static const long double L2E=1.442695040888963407359924681001892137L, LN2L=0.693147180559945309417232121458176568L;
typedef long double (*wf)(long double f);
static long double w_f2xm1(long double f){ return 1.0L+f2xm1(f); }
static long double w_half(long double f){ long double h=1.0L+f2xm1(f*0.5L); return h*h; }
static long double w_quarter(long double f){ long double h=1.0L+f2xm1(f*0.25L); h=h*h; return h*h; }
static long double w_m1(long double f){ return 2.0L*(1.0L+f2xm1(f-1.0L)); }
static long double w_m1b(long double f){ return 2.0L+2.0L*f2xm1(f-1.0L); }
static long double w_exp2l(long double f){ return exp2l(f); }
static long double w_expl(long double f){ return expl(f*LN2L); }
static long double w_powl(long double f){ return powl(2.0L,f); }
static long double w_twox(long double f){ long double u=f2xm1(f); return u+u+1.0L-u; } // silly
int main(){
  wf W[]={w_f2xm1,w_half,w_quarter,w_m1,w_m1b,w_exp2l,w_expl,w_powl};
  const char*N[]={"1+f2xm1(f)","(1+f2xm1(f/2))^2","^4","2(1+f2xm1(f-1))","2+2f2xm1(f-1)","exp2l","expl(f ln2)","powl"};
  int nv=8; static double A[200000],E[200000]; int n=0; char ah[64],eh[64],tag[64];
  while(scanf("%63s %63s %63s",ah,eh,tag)==3){ A[n]=strtod(ah,0); E[n]=strtod(eh,0); if(E[n]<0x1p-1022) continue; n++; }
  for(int v=0;v<nv;v++){ long ok=0;
    for(int i=0;i<n;i++){ long double t=L2E*(long double)A[i]; long double nn=floorl(t), f=t-nn; long double w=W[v](f); volatile long double r=fscale(w,nn); if((double)r==E[i]) ok++; }
    printf("%-22s %ld/%d\n",N[v],ok,n);}
}
