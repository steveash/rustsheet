// exact / within-1 / max-ulp / histogram of a raw-y (8,8)-slot N/D on a "yhex x x excelhex tag" file. gcc -O1 -mfpmath=387 maxerr.c -lm
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#define ST(v) ({volatile double _t=(v); _t;})
int main(int c,char**v){ double N[9],D[9]; for(int i=0;i<9;i++)N[i]=strtod(v[2+i],0); for(int i=0;i<9;i++)D[i]=strtod(v[11+i],0);
 FILE*f=fopen(v[1],"r"); char a[64],b[16],cc[16],d[64],e[64]; long tot=0,ok=0,w1=0,h[8]={0}; double mx=0;
 while(fscanf(f,"%s %s %s %s %s",a,b,cc,d,e)==5){ double y=strtod(a,0),g=strtod(d,0);
   double s=N[8]; for(int i=7;i>=0;i--){s=ST(s*y); s=ST(s+N[i]);} double t=D[8]; for(int i=7;i>=0;i--){t=ST(t*y); t=ST(t+D[i]);}
   double q=ST(s/t); double u=nextafter(g,10)-g; double er=fabs(q-g)/u; tot++; if(er==0)ok++; if(er<=1)w1++; if(er>mx)mx=er; int k=(int)er; if(k>7)k=7; h[k]++; }
 printf("exact %.2f%%  within1 %.2f%%  max %.0f ulps  hist:",100.0*ok/tot,100.0*w1/tot,mx); for(int k=0;k<8;k++) printf(" %ld",h[k]); printf("\n"); }
