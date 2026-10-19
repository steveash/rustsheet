// score raw-y Horner N/D (every op fl53(fl64)) on G.txt. usage: gsim G.txt n m N0..Nn D0..Dm ; gcc -O1 -mfpmath=387
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#define ST(v) ({volatile double _t=(v); _t;})
int main(int c,char**v){ int n=atoi(v[2]),m=atoi(v[3]); double N[20],D[20];
 for(int i=0;i<=n;i++)N[i]=strtod(v[4+i],0); for(int i=0;i<=m;i++)D[i]=strtod(v[5+n+i],0);
 FILE*f=fopen(v[1],"r"); char a[64],b[16],cc[16],d[64],e[64]; long tot=0,ok=0,b1=0,bins[10]={0},okb[10]={0};
 while(fscanf(f,"%s %s %s %s %s",a,b,cc,d,e)==5){ double y=strtod(a,0),g=strtod(d,0);
   double s=N[n]; for(int i=n-1;i>=0;i--){s=ST(s*y); s=ST(s+N[i]);} double t=D[m]; for(int i=m-1;i>=0;i--){t=ST(t*y); t=ST(t+D[i]);}
   double r=ST(s/t); int k=(int)((y-1)*10); tot++; bins[k]++; if(r==g){ok++;okb[k]++;} else if(r==nextafter(g,2)||r==nextafter(g,0)) b1++; }
 printf("exact %ld/%ld (%.1f%%) within1 %ld |",ok,tot,100.0*ok/tot,b1); for(int k=0;k<10;k++) printf(" %.0f",100.0*okb[k]/(bins[k]?bins[k]:1)); printf("\n"); }
