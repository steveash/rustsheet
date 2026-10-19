// evaluate raw-y Horner N/D (x87 fl53(fl64)) on an array.  gcc -O1 -mfpmath=387 -shared -fPIC -o libgeval.so geval.c
#define ST(v) ({volatile double _t=(v); _t;})
void geval(int np,const double*y,double*q,const double*N,const double*D){
  for(int p=0;p<np;p++){ double s=N[8]; for(int i=7;i>=0;i--){s=ST(s*y[p]); s=ST(s+N[i]);} double t=D[8]; for(int i=7;i>=0;i--){t=ST(t*y[p]); t=ST(t+D[i]);} q[p]=ST(s/t);} }
