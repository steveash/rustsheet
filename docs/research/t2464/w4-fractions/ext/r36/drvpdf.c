/* Loader-form beta density variants (R 3.6 nmath stirlerr), reads "a b x y" hex, prints variants %a. */
#include <stdio.h>
#include <math.h>
#include "nmath.h"
double stirlerr(double);
static double bd0s(double x, double np) {  /* R series/direct */
  if (fabs(x-np) < 0.1*(x+np)) { double v=(x-np)/(x+np), s=(x-np)*v, ej=2*x*v, s1; v=v*v;
    for (int j=1;j<1000;j++){ ej*=v; s1=s+ej/((j<<1)+1); if(s1==s) return s1; s=s1;} }
  return x*log(x/np)+np-x; }
static double bd0d(double x, double np) { return x*log(x/np)+np-x; }
static double bd0l(double x, double np) { return x*log1p((x-np)/np)+np-x; }
static double draw(double k,double n,double p,double q,int v){
  double (*bd)(double,double) = (v%3==0)?bd0s:(v%3==1)?bd0d:bd0l;
  double lc = stirlerr(n) - stirlerr(k) - stirlerr(n-k) - bd(k,n*p) - bd(n-k,n*q);
  double lf = (v/3==0) ? M_LN_2PI + log(k) + log1p(-k/n) : log(6.283185307179586476925286766559) + log(k) + log(n-k) - log(n);
  return lc - 0.5*lf; }
int main(void){ double a,b,x,y; while(scanf("%la %la %la %la",&a,&b,&x,&y)==4){
  for(int v=0;v<6;v++){ double l = draw(a-1,a+b-2,x,y,v); printf("%a %a ", exp(log(a+b-1)+l), (a+b-1)*exp(l)); }
  printf("\n"); } return 0; }
