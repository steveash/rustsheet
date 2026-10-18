#include "common.h"
/* apply COEF k c1off d2 d3 ... OUT : c_{j+2} += d_j * u2 / 0.25^j, u2=ulp(c2); then score all */
int main(int argc,char**argv){ loadpts(); loadcoef(argv[1]); int k=atoi(argv[2]); int off=atoi(argv[3]); double*c=k?Qd:Pd; int n=k?nq:np;
 for(int i=0;i<abs(off);i++) c[1]=nextafter(c[1],off>0?9:-9);
 double u2=nextafter(c[2],9)-c[2]; int nd=argc-5;
 for(int j=0;j<nd;j++){ double d=atof(argv[4+j]); if(j==0) d=nearbyint(d); LD nv=(LD)c[2+j]+(LD)d*(LD)u2/powl(0.25L,j); c[2+j]=(double)nv; }
 savecoef(argv[argc-1]);
 long h=0,hf[16]={0},nf[16]={0}; for(long i=0;i<NP;i++){ double x=P_[i].x; double z=st((LD)x*x); int ok=fin(x,horner(np,Pd,z,0),horner(nq,Qd,z,0))==P_[i].y; h+=ok; hf[P_[i].f]+=ok; nf[P_[i].f]++; }
 printf("score %ld/%ld  per-file miss:",h,NP); for(int f=0;f<12;f++) printf(" %ld",nf[f]-hf[f]); printf("\n"); (void)n; }
