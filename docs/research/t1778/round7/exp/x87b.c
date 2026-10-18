#include <stdio.h>
#include <stdlib.h>
#include <math.h>
static void setpc(unsigned short pc){ unsigned short cw; __asm__ volatile("fnstcw %0":"=m"(cw)); cw=(cw&~0x0F00)|(pc<<8); __asm__ volatile("fldcw %0"::"m"(cw)); }
static void setrc(unsigned short rc){ unsigned short cw; __asm__ volatile("fnstcw %0":"=m"(cw)); cw=(cw&~0x0C00)|(rc<<10); __asm__ volatile("fldcw %0"::"m"(cw)); }
static long double f2xm1(long double f){ long double u; __asm__ volatile("f2xm1":"=t"(u):"0"(f)); return u; }
static long double fscale(long double w,long double n){ long double r; __asm__ volatile("fscale":"=t"(r):"0"(w),"u"(n)); return r; }
static long double frndint(long double t){ long double r; __asm__ volatile("frndint":"=t"(r):"0"(t)); return r; }
static long double fldl2e(void){ long double r; __asm__ volatile("fldl2e":"=t"(r)); return r; }
volatile long double sink;
// generic: pcT precision for t product, pcA for 1+u add; split mode 0 nint 1 floor 2 trunc ; mode for final
static double gen(double a,int pcT,int pcA,int split,int addmode){
  setpc(pcT);
  long double L=fldl2e();
  long double t=L*(long double)a; sink=t;
  setpc(3);
  long double n;
  if(split==0) n=frndint(t); else { setrc(split==1?1:3); n=frndint(t); setrc(0);} 
  long double f=t-n;
  long double u=f2xm1(f);
  setpc(pcA);
  long double w,r;
  if(addmode==0){ w=u+1.0L; sink=w; setpc(3); r=fscale(w,n);} 
  else { long double s=fscale(u,n), p=fscale(1.0L,n); r=s+p; sink=r; setpc(3);} 
  double d=(double)r; return d;
}
int main(int argc,char**argv){
  static double A[200000],E[200000]; int n=0; char ah[64],eh[64],tag[64];
  while(scanf("%63s %63s %63s",ah,eh,tag)==3){ A[n]=strtod(ah,0); E[n]=strtod(eh,0); if(E[n]==0) continue; n++; }
  int pcs[2]={3,2};
  for(int pT=0;pT<2;pT++)for(int pA=0;pA<2;pA++)for(int sp=0;sp<3;sp++)for(int am=0;am<2;am++){
    long ok=0; for(int i=0;i<n;i++) if(gen(A[i],pcs[pT],pcs[pA],sp,am)==E[i]) ok++;
    printf("pcT=%d pcA=%d split=%d add=%d : %ld/%d\n",pcs[pT]==3?64:53,pcs[pA]==3?64:53,sp,am,ok,n);
  }
}
