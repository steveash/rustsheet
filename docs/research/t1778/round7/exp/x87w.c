#include <stdio.h>
#include <stdlib.h>
#include <math.h>
static void setrc(unsigned short rc){ unsigned short cw; __asm__ volatile("fnstcw %0":"=m"(cw)); cw=(cw&~0x0F00)|0x0300|(rc<<10); __asm__ volatile("fldcw %0"::"m"(cw)); }
static long double f2xm1(long double f){ long double u; __asm__ volatile("f2xm1":"=t"(u):"0"(f)); return u; }
static long double frndint(long double t){ long double r; __asm__ volatile("frndint":"=t"(r):"0"(t)); return r; }
static long double fldl2e(void){ long double r; __asm__ volatile("fldl2e":"=t"(r)); return r; }
int main(){ char ah[64],eh[64],tag[64]; setrc(0);
  while(scanf("%63s %63s %63s",ah,eh,tag)==3){ double a=strtod(ah,0);
    long double t=fldl2e()*(long double)a;
    long double n0=frndint(t); setrc(1); long double n1=frndint(t); setrc(0);
    long double f0=t-n0,f1=t-n1; long double u0=f2xm1(f0),u1=f2xm1(f1);
    long double w0=u0+1.0L,w1=u1+1.0L;
    printf("%La %La %La %La %La %La %d %d\n",t,f0,u0,f1,u1,w1,(int)n0,(int)n1);
  }}
