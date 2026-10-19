// for each line "yhex excelhex resid" output "yhex resid_sim" with resid_sim = resid + (model-excel)/2^-53
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#define ST(v) ({volatile double _t=(v); _t;})
int main(int c,char**v){ double N[9],D[9]; for(int i=0;i<9;i++)N[i]=strtod(v[3+i],0); for(int i=0;i<9;i++)D[i]=strtod(v[12+i],0);
 FILE*f=fopen(v[1],"r"),*o=fopen(v[2],"w"); char a[64],b[64]; double r;
 while(fscanf(f,"%s %s %lf",a,b,&r)==3){ double y=strtod(a,0),g=strtod(b,0);
   double s=N[8]; for(int i=7;i>=0;i--){s=ST(s*y); s=ST(s+N[i]);} double t=D[8]; for(int i=7;i>=0;i--){t=ST(t*y); t=ST(t+D[i]);}
   double q=ST(s/t); fprintf(o,"%s %.6f\n",a,r+(q-g)/ldexp(1,-53)); } }
