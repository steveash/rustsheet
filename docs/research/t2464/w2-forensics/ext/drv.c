#include <stdio.h>
#include <stdlib.h>
void bratio(double a, double b, double x, double y, double *w, double *w1, int *ierr, int log_p);
int main(){ double a,b,x,y,w,w1; int ierr;
 while (scanf("%la %la %la %la",&a,&b,&x,&y)==4){ bratio(a,b,x,y,&w,&w1,&ierr,0); printf("%a %a %d\n",w,w1,ierr);} return 0;}
