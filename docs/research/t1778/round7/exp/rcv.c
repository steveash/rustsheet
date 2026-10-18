#include <stdio.h>
#include <stdlib.h>
static void setrc(unsigned short rc){ unsigned short cw; __asm__ volatile("fnstcw %0":"=m"(cw)); cw=(cw&~0x0F00)|0x0300|(rc<<10); __asm__ volatile("fldcw %0"::"m"(cw)); }
static long double f2xm1(long double f){ long double u; __asm__ volatile("f2xm1":"=t"(u):"0"(f)); return u; }
static long double fscale(long double w,long double n){ long double r; __asm__ volatile("fscale":"=t"(r):"0"(w),"u"(n)); return r; }
static long double frndint(long double t){ long double r; __asm__ volatile("frndint":"=t"(r):"0"(t)); return r; }
static long double fsub(long double a,long double b){ long double r; __asm__ volatile("fsubp":"=t"(r):"0"(a),"u"(b):"st(1)"); return r; }
static long double fadd1(long double a){ long double r; __asm__ volatile("fld1\n\tfaddp":"=t"(r):"0"(a)); return r; }
static const long double L2E=1.442695040888963407359924681001892137L;
// stage at which RC=RCX is restored to nearest: 1 after frndint, 2 after fsub, 3 after f2xm1, 4 after fadd, 5 after fscale
static double run(double a,int rcx,int stage,int rcf){
  setrc(0); long double t=L2E*(long double)a;
  setrc(rcx); long double n=frndint(t); if(stage<=1) setrc(0);
  long double f=fsub(t,n); if(stage<=2) setrc(0);
  long double u=f2xm1(f); if(stage<=3) setrc(0);
  long double w=fadd1(u); if(stage<=4) setrc(0);
  long double r=fscale(w,n); setrc(rcf);
  volatile double d=(double)r; setrc(0); return d;
}
int main(){ static double A[200000],E[200000]; int n=0; char ah[64],eh[64],tag[64];
  while(scanf("%63s %63s %63s",ah,eh,tag)==3){ A[n]=strtod(ah,0); E[n]=strtod(eh,0); if(E[n]<0x1p-1022) continue; n++; }
  for(int rcx=0;rcx<4;rcx++) for(int st=1;st<=5;st++){ if(rcx==0&&st>1) continue; long ok=0;
    for(int i=0;i<n;i++) if(run(A[i],rcx,st,0)==E[i]) ok++;
    printf("RC=%d during frndint..stage%d : %ld/%d\n",rcx,st,ok,n);}
}
