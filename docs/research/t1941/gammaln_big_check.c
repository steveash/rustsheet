#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long double L;
static L S(L v){ volatile double t=(double)v; return t; }
static L ln_excel(L x){ L r; __asm__("fldln2\n\tfxch\n\tfyl2x":"=t"(r):"0"(x)); return S(r); }
static const double SQ=0.9189385332046727;
static const double BC[13]={1.0/12,-1.0/360,1.0/1260,-1.0/1680,1.0/1188,-691.0/360360,1.0/156,-3617.0/122400,43867.0/244188,-174611.0/125400,77683.0/5796,-236364091.0/1506960,657931.0/300};
static int NT=10; static double CC;
static L series(L y){ L z=S(1/y), z2=S(z*z), r=BC[NT-1]; for(int i=NT-2;i>=0;i--) r=S(S(r*z2)+BC[i]); return S(r*z); }
static double gammaln_big(double x){ L y=x, corr=getenv("USEC")?CC:ln_excel(y), a=S(y-0.5L), b=S(a*corr), d=S(b-y), e=S(d+SQ); return S(e+series(y)); }
int main(int argc,char**argv){
  if(getenv("NT")) NT=atoi(getenv("NT"));
  double xmin=atof(argv[2]), xmax=atof(argv[3]);
  static char nm[256][80]; static long n[256],ok[256]; int nc=0;
  FILE*f=fopen(argv[1],"r"); char a[64],b[64],c[64],d[64],e[80];
  while(fscanf(f,"%s %s %s %s %79s",a,b,c,d,e)==5){
    double x=strtod(a,0); if(x<xmin||x>xmax||b[0]=='n') continue; double g=strtod(b,0); CC=strtod(c,0);
    int k; for(k=0;k<nc;k++) if(!strcmp(nm[k],e)) break; if(k==nc) strcpy(nm[nc++],e);
    double m=gammaln_big(x); n[k]++; if(m==g) ok[k]++; else if(getenv("V")) printf("miss %a model %a excel %a %s\n",x,m,g,e); }
  long N=0,O=0; for(int k=0;k<nc;k++){ printf("%-24s %6ld/%6ld\n",nm[k],ok[k],n[k]); N+=n[k]; O+=ok[k]; }
  printf("TOTAL %ld/%ld\n",O,N); }
