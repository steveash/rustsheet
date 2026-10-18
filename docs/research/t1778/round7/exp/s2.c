#include <stdio.h>
#include <stdlib.h>
static long double f2xm1(long double f){ long double u; __asm__ volatile("f2xm1":"=t"(u):"0"(f)); return u; }
static long double fscale(long double w,long double n){ long double r; __asm__ volatile("fscale":"=t"(r):"0"(w),"u"(n)); return r; }
static const long double L2E=1.442695040888963407359924681001892137L;
static const long double S2=1.414213562373095048801688724209698079L;
static const long double IS2=0.707106781186547524400844362104849039L;
static double run(double a,int v,double thr){
  long double t=L2E*(long double)a; long double n=__builtin_floorl(t), f=t-n, w;
  if(v==0){ w=1.0L+f2xm1(f); }
  else if(v==1){ if(f>thr){ long double u=f2xm1(f-0.5L); w=(1.0L+u)*S2; } else w=1.0L+f2xm1(f); }
  else if(v==2){ if(f>thr){ long double u=f2xm1(f-0.5L); w=u*S2+S2; } else w=1.0L+f2xm1(f); }
  else if(v==3){ // nint split, negative half uses 2^(f+0.5)/sqrt2
    long double nn=__builtin_rintl(t), g=t-nn; n=nn;
    if(g<0){ long double u=f2xm1(g+0.5L); w=(1.0L+u)/S2; } else w=1.0L+f2xm1(g); }
  else { long double nn=__builtin_rintl(t), g=t-nn; n=nn;
    if(g<0){ long double u=f2xm1(g+0.5L); w=(1.0L+u)*IS2; } else w=1.0L+f2xm1(g); }
  volatile long double r=fscale(w,n); return (double)r;
}
int main(){ static double A[200000],E[200000]; int n=0; char ah[64],eh[64],tag[64];
  while(scanf("%63s %63s %63s",ah,eh,tag)==3){ A[n]=strtod(ah,0); E[n]=strtod(eh,0); if(E[n]<0x1p-1022) continue; n++; }
  const char*N[]={"floor plain","floor f>.5: (1+u)*S2","floor f>.5: u*S2+S2","nint g<0: (1+u)/S2","nint g<0: (1+u)*IS2"};
  for(int v=0;v<5;v++){ long ok=0; for(int i=0;i<n;i++) if(run(A[i],v,0.5)==E[i]) ok++; else if(v==1) fprintf(stderr,"%a %a %s\n",A[i],E[i],""); printf("%-28s %ld/%d\n",N[v],ok,n);} }
