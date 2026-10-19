/* Copied by T-1916 from branch agent/research/T-1778-kernel-map at 59c2c3724a3c (path docs/research/t1778/round9/mid_check.c), unchanged below this line. */
#include <stdio.h>
#include <stdlib.h>
typedef long double LD;
static double st(LD v){volatile double d=(double)v;return d;}
static double horner(const double*c,int n,double x){double s=c[0];for(int i=1;i<n;i++){volatile LD t=(LD)s*x;double tt=st(t);volatile LD u=(LD)tt+c[i];s=st(u);}return s;}
/* usage: mid coef.txt pts.txt ; coef file: "N k c..." line and "D k c..." line, highest degree first */
int main(int argc,char**argv){double N[32],D[32];int nn=0,nd=0;char t[8];FILE*cf=fopen(argv[1],"r");
 for(int L=0;L<2;L++){int k;fscanf(cf,"%7s %d",t,&k);double*c=t[0]=='N'?N:D;if(t[0]=='N')nn=k;else nd=k;for(int i=0;i<k;i++){char h[64];fscanf(cf,"%63s",h);c[i]=strtod(h,0);}}
 FILE*f=fopen(argv[2],"r");char a[64],b[64],e[64],nm[128];long n=0,ok=0,okR=0;
 while(fscanf(f,"%63s %63s %63s %127s",a,b,e,nm)==4){double x=strtod(a,0),y=strtod(b,0),E=strtod(e,0);
  double p=horner(N,nn,x),q=horner(D,nd,x);volatile LD r=(LD)p/q;double R=st(r);volatile LD m=(LD)E*R;double ym=st(m);
  n++;if(ym==y)ok++;}
 printf("TOTAL %ld/%ld (%.3f%%)\n",ok,n,100.0*ok/n);}
