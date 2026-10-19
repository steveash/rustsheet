#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
typedef long double L;
static inline L f2xm1(L x){L r; __asm__("f2xm1":"=t"(r):"0"(x)); return r;}
static inline L fyl2x(L x,L y){L r; __asm__("fyl2x":"=t"(r):"0"(x),"u"(y):"st(1)"); return r;}
static inline L fscale(L x,L e){L r; __asm__("fscale":"=t"(r):"0"(x),"u"(e)); return r;}
static inline L frndint(L x){L r; __asm__("frndint":"=t"(r):"0"(x)); return r;}
static inline L fldl2e(void){L r; __asm__("fldl2e":"=t"(r)); return r;}
static inline L fldln2(void){L r; __asm__("fldln2":"=t"(r)); return r;}
#define STL(v) ({volatile L _t=(v); _t;})
#define ST(v) ({volatile double _t=(v); _t;})
static double xexp(double a){ L t=STL((L)a*fldl2e()); L n=frndint(t); L f=STL(t-n); L u=f2xm1(fabsl(f)); L w=STL(1.0L+u); if(f<0) w=STL(1.0L/w); volatile double r=(double)fscale(w,n); return r; }
static double LNX; static int USEC=0;
static double xln(double x){ if(USEC) return LNX; L r=fyl2x((L)x,fldln2()); volatile double d=(double)r; return d;}
static const double B[10]={1.0/12,-1.0/360,1.0/1260,-1.0/1680,1.0/1188,-691.0/360360,1.0/156,-3617.0/122400,43867.0/244188,-174611.0/125400};
static double gamma_big(double x){
  double h=ST(0.5*x), nn=floor(h); long M=(long)nn; double c2=ST(h-nn);
  double r=1.0,bb=x; int first=1;
  while(M){ if(M&1){ if(first) r=bb; else r=ST(r*bb); first=0;} M>>=1; if(M) bb=ST(bb*bb);}
  double v=ST(r*xexp(ST(c2*xln(x))));
  double w1=ST(1.0/x), z2=ST(w1*w1), rr=B[9]; for(int k=8;k>=0;k--){ rr=ST(rr*z2); rr=ST(rr+B[k]); }
  double W=xexp(ST(rr*w1));
  double t=ST(xexp(-x)*ST(sqrt(w1))); t=ST(t*v); t=ST(t*v); t=ST(t*2.50662827463100050242); return ST(t*W); }
int main(int argc,char**argv){ USEC=getenv("USEC")!=0; FILE*f=fopen(argv[1],"r"); char a[64],b[64],c[64],d[64],e[80]; long n=0,ok=0;
 while(fscanf(f,"%s %s %s %s %79s",a,b,c,d,e)==5){ double x=strtod(a,0); if(d[0]=='n'||x<10||x>=171.624376956302725) continue;
  if(x==floor(x)&&x<89) continue; if(x-floor(x)==0.5) continue; if(c[0]=='n'&&USEC) continue; LNX=strtod(c,0);
  double g=strtod(d,0); n++; if(gamma_big(x)==g) ok++; }
 printf("TOTAL %ld/%ld\n",ok,n); }
