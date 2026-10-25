#include "toms708_nochk.c"
int main(void){ double a,b,x,y,w,w1; int ierr;
 while(scanf("%la %la %la %la",&a,&b,&x,&y)==4){ bratio(a,b,x,y,&w,&w1,&ierr,0); printf("%a %a %d\n",w,w1,ierr);} return 0;}
