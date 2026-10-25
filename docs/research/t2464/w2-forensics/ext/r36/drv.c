#include <stdio.h>
#include <string.h>
#include "nmath.h"
int main(void){ char fn[32]; double a,b,x,y;
 while (scanf("%31s %la %la %la %la", fn,&a,&b,&x,&y)==5){ double r=NAN;
  if(!strcmp(fn,"dbeta")) r=dbeta(x,a,b,0);
  else if(!strcmp(fn,"lbeta")) r=lbeta(a,b);
  else if(!strcmp(fn,"lgamma")) r=lgammafn(a);
  else if(!strcmp(fn,"stirlerr")) r=stirlerr(a);
  printf("%a\n", r);} return 0;}
