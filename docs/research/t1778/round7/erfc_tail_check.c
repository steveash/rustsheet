#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long double LD;
static double st(LD v){volatile double d=(double)v;return d;}
static const double N[9]={0x1.4dc756d76135p-2,0x1.6ca0f8ecfc798p+1,0x1.0191f83634b97p+4,0x1.d936f5138efd9p+5,0x1.396f0fc8e5c9cp+7,0x1.26c409d384172p+8,0x1.80fb2b5716d18p+8,0x1.3da266cd039ep+8,0x1.000aebf22de16p+7};
static const double D[10]={0x1.27cdbb941913dp-1,0x1.4324d3ef0f238p+2,0x1.cd27390103553p+4,0x1.ad794d2e2c325p+6,0x1.23c050f5fc4a3p+8,0x1.1e2d955b8fd9dp+9,0x1.93da0f9daba69p+9,0x1.850e3f0a8dbb6p+9,0x1.ce328ffeb78acp+8,0x1p+7};
static double horner(const double*c,int n,double x){double s=c[0];for(int i=1;i<n;i++){volatile LD t=(LD)s*x;double tt=st(t);volatile LD u=(LD)tt+c[i];s=st(u);}return s;}
int main(){FILE*f=fopen("pts.txt","r");char a[64],b[64],e[64],nm[128];long n=0,ok=0;
 while(fscanf(f,"%63s %63s %63s %127s",a,b,e,nm)==4){double x=strtod(a,0),y=strtod(b,0),E=strtod(e,0);
  double nn=horner(N,9,x),dd=horner(D,10,x); volatile LD q=(LD)nn/dd; double R=st(q); volatile LD p=(LD)E*R; double ym=st(p);
  n++; if(ym==y) ok++; else printf("MISS %s x=%a y=%a model=%a\n",nm,x,y,ym);}
 printf("TOTAL %ld/%ld\n",ok,n);}
