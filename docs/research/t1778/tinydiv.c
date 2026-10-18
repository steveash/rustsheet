/* T-1778: tiny-x erf as fl(fl(x*A)/Q(0)) with an x87 product store and divide. Run from the directory where mk_tiny.py wrote tiny.txt; build: cc -O0 tinydiv.c -o tinydiv */
#include <stdio.h>
#include <stdlib.h>
int main(){ FILE*f=fopen("tiny.txt","r"); char a[80],b[80],c[80]; int n=0,ok80=0,okd=0;
 double A=0x1.1df5e70c7956bp+0, D0=0x1.fada15778ece6p-1;
 char line[400];
 while(fgets(line,400,f)){ int k=sscanf(line,"%79s %79s %79s",a,b,c); if(k<2) continue; double x=strtod(k==3?b:a,0), e=strtod(k==3?c:b,0);
   volatile long double p=(long double)x*A; volatile double co=(double)p; volatile long double q=(long double)co/D0; double r80=(double)q; double rd=co/D0;
   n++; ok80+=(r80==e); okd+=(rd==e);} printf("n %d  80-bit div %d  double div %d\n",n,ok80,okd); }
