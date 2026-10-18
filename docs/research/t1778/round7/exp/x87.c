// Real-hardware x87 EXP variants. Reads "a_hex E_hex tag" lines, prints per-variant match counts,
// writes mismatches of variant chosen by argv[1] to stdout-file.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
static void setcw(unsigned short pc){ unsigned short cw; __asm__ volatile("fnstcw %0":"=m"(cw)); cw=(cw&~0x0F00)|(pc<<8); __asm__ volatile("fldcw %0"::"m"(cw)); }
// V1: FLDL2E; FMUL a; FLD st; FRNDINT; FSUB st(1),st ; F2XM1; FLD1; FADD; FSCALE; FSTP double
static double v_f2xm1(double a){
  long double r;
  __asm__ volatile(
   "fldl %1\n\t"      // a
   "fldl2e\n\t"       // l2e a
   "fmulp\n\t"        // t
   "fld %%st(0)\n\t"  // t t
   "frndint\n\t"      // n t
   "fxch\n\t"         // t n
   "fsub %%st(1),%%st\n\t" // f n
   "f2xm1\n\t"        // u n
   "fld1\n\t" "faddp\n\t" // w n
   "fscale\n\t"       // r n
   "fstp %%st(1)\n\t"
   : "=t"(r) : "m"(a));
  return (double)r;
}
// V2: same but fraction from truncation toward -inf (floor) via RC change -> f in [0,1)
static double v_f2xm1_floor(double a){
  long double t=(long double)a*1.442695040888963407359924681001892137L; // L2E rounded to 64 by compiler
  long double n=floorl(t), f=t-n, u;
  __asm__ volatile("f2xm1":"=t"(u):"0"(f));
  long double w=u+1.0L, r;
  __asm__ volatile("fscale":"=t"(r):"0"(w),"u"(n));
  return (double)r;
}
// V3: half-split with F2XM1 on f/2 squared? (some libs: 2^f = (2^(f/2))^2) -- skip
// V4: glibc exp (correct-ish), expl rounded
static double v_expl(double a){ return (double)expl((long double)a); }
static double v_exp(double a){ return exp(a); }
// V5: F2XM1 result with u computed then fscale directly on u then add 2^n (x87 ld): r=2^n + u*2^n
static double v_f2xm1_sep(double a){
  long double t=(long double)a*1.442695040888963407359924681001892137L;
  long double n=nearbyintl(t), f=t-n, u;
  __asm__ volatile("f2xm1":"=t"(u):"0"(f));
  long double s, one=1.0L;
  __asm__ volatile("fscale":"=t"(s):"0"(u),"u"(n));
  long double p2; __asm__ volatile("fscale":"=t"(p2):"0"(one),"u"(n));
  return (double)(s+p2);
}
typedef double (*fn)(double);
int main(int argc,char**argv){
  setcw(3); // 64-bit precision
  fn F[]={v_f2xm1,v_f2xm1_floor,v_expl,v_exp,v_f2xm1_sep}; const char*N[]={"f2xm1_nint","f2xm1_floor","expl","glibc_exp","f2xm1_sep"};
  int nv=5; long ok[8]={0}; long n=0; char ah[64],eh[64],tag[64];
  int show=argc>1?atoi(argv[1]):-1;
  while(scanf("%63s %63s %63s",ah,eh,tag)==3){
    double a=strtod(ah,0),E=strtod(eh,0); n++;
    for(int i=0;i<nv;i++){ double r=F[i](a); if(r==E) ok[i]++; else if(i==show) printf("MIS %a %a %a %s\n",a,E,r,tag);}
  }
  for(int i=0;i<nv;i++) fprintf(stderr,"%-12s %ld/%ld\n",N[i],ok[i],n);
}
