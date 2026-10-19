// shared lib: load G points once ("yhex x x excelhex tag"), score raw-y Horner N/D. gcc -O1 -mfpmath=387 -shared -fPIC -o libgscore.so gscore.c -lm
#include <stdio.h>
#include <stdlib.h>
#define ST(v) ({volatile double _t=(v); _t;})
static double *Y,*V; static long NP=0;
int gload(const char*fn,int stride){ FILE*f=fopen(fn,"r"); char a[64],b[16],c[16],d[64],e[64]; long cap=4000000,k=0;
  Y=malloc(cap*8);V=malloc(cap*8); while(fscanf(f,"%s %s %s %s %s",a,b,c,d,e)==5){ if((k++)%stride) continue; if(NP>=cap) break; Y[NP]=strtod(a,0);V[NP]=strtod(d,0);NP++;} fclose(f); return NP;}
long gscore(int n,int m,const double*N,const double*D){ long ok=0;
  for(long p=0;p<NP;p++){ double y=Y[p]; double s=N[n]; for(int i=n-1;i>=0;i--){s=ST(s*y); s=ST(s+N[i]);} double t=D[m]; for(int i=m-1;i>=0;i--){t=ST(t*y); t=ST(t+D[i]);}
    if(ST(s/t)==V[p]) ok++; } return ok;}
double gabserr(int n,int m,const double*N,const double*D){ double acc=0;
  for(long p=0;p<NP;p++){ double y=Y[p]; double s=N[n]; for(int i=n-1;i>=0;i--){s=ST(s*y); s=ST(s+N[i]);} double t=D[m]; for(int i=m-1;i>=0;i--){t=ST(t*y); t=ST(t+D[i]);}
    double q=ST(s/t); double u=__builtin_nextafter(V[p],10.0)-V[p]; double e=(q-V[p])/u; acc+= e<0?-e:e; } return acc/NP;}
