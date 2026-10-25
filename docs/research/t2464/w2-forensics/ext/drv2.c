/* Expose TOMS 708 internals: reads "fn a b x y" lines (fn name, hex doubles), prints hex result. */
#include "toms708.c"
#include <string.h>
#include <stdlib.h>
int main(void){ char fn[32]; double a,b,x,y; double eps = 1e-15;
 while (scanf("%31s %la %la %la %la", fn,&a,&b,&x,&y)==5){ double r=NAN; int ierr=0; double w,w1;
  if(!strcmp(fn,"bratio")){ bratio(a,b,x,y,&w,&w1,&ierr,0); r=w; }
  else if(!strcmp(fn,"bratio1")){ bratio(a,b,x,y,&w,&w1,&ierr,0); r=w1; }
  else if(!strcmp(fn,"brcomp")) r=brcomp(a,b,x,y,0);
  else if(!strcmp(fn,"betaln")) r=betaln(a,b);
  else if(!strcmp(fn,"gamln")) r=gamln(a);
  else if(!strcmp(fn,"algdiv")) r=algdiv(a,b);
  else if(!strcmp(fn,"bcorr")) r=bcorr(a,b);
  else if(!strcmp(fn,"bpser")) r=bpser(a,b,x,eps,0);
  else if(!strcmp(fn,"bfrac")) { double lambda = (a > b) ? (a+b)*y - b : a - (a+b)*x; r=bfrac(a,b,x,y,lambda,eps*15.,0);} 
  else if(!strcmp(fn,"dbeta_brc")) r=brcomp(a,b,x,y,0)/(x*y);
  printf("%a\n", r);} return 0;}
