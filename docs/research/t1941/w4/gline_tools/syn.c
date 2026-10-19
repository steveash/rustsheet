// synthetic Horner-in-y N/D (W1 near-1 N3/D3) residuals on the same y grids; "./syn s" prints stage u,v at y=1
// build: gcc -O1 -mfpmath=387 -o syn syn.c -lquadmath -lm ; ./syn e3res.txt out h | ./syn w104res.txt out d
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <quadmath.h>
#define ST(v) ({volatile double _t=(v); _t;})
static double N[11],D[11]; static int nn=10,nd=10;
static const char*Ns[]={"0","-0x1.59f97cb2595b3p-11","-0x1.92587e5cc6b87p-6","-0x1.8558f6ae39a6dp-3","-0x1.d3f13713066a8p-2","-0x1.4c00f8658473ap-2","0x1.59e7ca69322a9p-5","0x1.6d0d491d31729p-4","0x1.3038d89b42949p-6","0x1.c3aa7b639fa87p-11","0x1.337ad32ea909fp-19"};
static const char*Ds[]={"0","0x1p-13","0x1.f168c3e77cdcfp-8","0x1.72eb273a4ee66p-4","0x1.6b0b4a0843b69p-2","0x1.18e8390e5354fp-1","0x1.6e6b1d23cccc3p-2","0x1.8a4a651d4508dp-4","0x1.35398ecbcfb5dp-7","0x1.ebfe99899feccp-13","-0x1.4461de6b330d3p-31"};
static double model(double y){ double n=N[nn]; for(int i=nn-1;i>=0;i--){n=ST(n*y); n=ST(n+N[i]);} double d=D[nd]; for(int i=nd-1;i>=0;i--){d=ST(d*y); d=ST(d+D[i]);} return ST(n/d);}
static __float128 ex(double y){ __float128 Y=y,n=N[nn],d=D[nd]; for(int i=nn-1;i>=0;i--) n=n*Y+N[i]; for(int i=nd-1;i>=0;i--) d=d*Y+D[i]; return n/d;}
static void stages(const char*nm,double*C,int deg){
  double m=C[deg],mp=0; for(int i=deg-1;i>=0;i--){ int e; frexp(m,&e); e-=1; double u=ldexp(m,-e), v=ldexp(mp,-e);
    printf("%s mult->c%d  M=%.10g u=%.8f frac=%.8f v=%.6f\n",nm,i,m,u,u-floor(u),v); double nm2=m+C[i]; mp=mp+m; m=nm2; }}
int main(int c,char**v){ for(int i=0;i<11;i++){N[i]=strtod(Ns[i],0);D[i]=strtod(Ds[i],0);}
 if(c>1 && v[1][0]=='s'){stages("N",N,nn);stages("D",D,nd);return 0;}
 FILE*f=fopen(v[1],"r"),*o=fopen(v[2],"w"); char a[64],b[64],cc[64]; int hex=v[3][0]=='h';
 while(1){ double y; if(hex){ if(fscanf(f,"%s %s %s",a,b,cc)!=3)break; y=strtod(a,0);} else { if(fscanf(f,"%s %s",a,b)!=2)break; y=strtod(a,0);}
   double m=model(y); __float128 e=ex(y); double r=(double)((m-e)/ (__float128)ldexp(1,-53));
   if(hex) fprintf(o,"%s %a %.6f\n",a,m,r); else fprintf(o,"%.17g %.6f\n",y,r); }
}
