// stage blame: per stage, near-tie rate (|frac-0.5|<0.05 of exact pre-rounding value) at mismatches vs matches. gcc -O1 -mfpmath=387 blame.c -lquadmath -lm
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <quadmath.h>
#define ST(v) ({volatile double _t=(v); _t;})
static double tiedist(__float128 ex, double rounded){ double u=nextafter(fabs(rounded),INFINITY)-fabs(rounded); __float128 d=(ex-(__float128)rounded)/u; double dd=(double)d; return 0.5-fabs(dd);}
int main(int c,char**v){ double N[9],D[9]; for(int i=0;i<9;i++)N[i]=strtod(v[2+i],0); for(int i=0;i<9;i++)D[i]=strtod(v[11+i],0);
 FILE*f=fopen(v[1],"r"); char a[64],b[16],cc[16],d[64],e[64]; int NS=33; long cnt[2][40]={{0}},near[2][40]={{0}};
 while(fscanf(f,"%s %s %s %s %s",a,b,cc,d,e)==5){ double y=strtod(a,0),g=strtod(d,0); double td[40]; int k=0;
   double s=N[8]; for(int i=7;i>=0;i--){ double p=ST(s*y); td[k++]=tiedist((__float128)s*y,p); double s2=ST(p+N[i]); td[k++]=tiedist((__float128)p+N[i],s2); s=s2;}
   double t=D[8]; for(int i=7;i>=0;i--){ double p=ST(t*y); td[k++]=tiedist((__float128)t*y,p); double t2=ST(p+D[i]); td[k++]=tiedist((__float128)p+D[i],t2); t=t2;}
   double q=ST(s/t); td[k++]=tiedist((__float128)s/t,q); int mis=(q!=g);
   for(int j=0;j<k;j++){ cnt[mis][j]++; if(td[j]<0.05) near[mis][j]++; } }
 for(int j=0;j<NS;j++){ double r1=(double)near[1][j]/cnt[1][j], r0=(double)near[0][j]/cnt[0][j];
   printf("stage %2d %s c%d %s  mismatch %.4f match %.4f ratio %.2f\n",j,j<16?"N":(j<32?"D":"Q"),7-(j%16)/2,(j%2==0)?"prod":"sum",r1,r0,r0>0?r1/r0:0); } }
