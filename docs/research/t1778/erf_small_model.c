/* T-1778: candidate reconstruction of Excel's small-x erf kernel (z = x*x <= 6E-9), for research only.
 *
 * Model (constants fitted from excel-graph probes kmyst-erf-*; see docs/excel-semantics/statistics.md):
 *   hi = fl(A + a1*z)                 coarse factor, rounded to double
 *   W  = fl(1 - BETA*z)               a value just below 1, on the 2^-53 grid
 *   lo = B0 + K*(W - 1)               fine factor
 *   erf = fl(fl(x*hi) + x*lo)         coarse product stored; final sum in 80-bit x87, then rounded
 * It reproduces about 99.6% of 67,048 probe points bit for bit. The rest are near ties.
 * Build: gcc -O2 -ffp-contract=off erf_small_model.c -lm ; run with a file of "id x_hex excel_hex" lines.
 */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#define MAXP 120000
double X[MAXP], E[MAXP]; int N=0;
double A=1.1170334248975802, a1=0.1406813111, K=1.1366973292490381, BETA=0.4546572618, B0=0.011345742197932506;
typedef long double LD;
int main(int argc,char**argv){
  FILE*f=fopen(argv[1],"r"); char a[64],b[64]; int id;
  while(fscanf(f,"%d %63s %63s",&id,a,b)==3){X[N]=strtod(a,0); E[N]=strtod(b,0); if(X[N]*X[N]<=6e-9) N++;}
  if(argc>2) B0=strtod(argv[2],0);
  int c[8]={0};
  for(int i=0;i<N;i++){ double x=X[i];
    volatile double z=x*x;                     /* z stored as double */
    volatile double hi=(double)((LD)A+(LD)a1*z);
    volatile double W=(double)(1.0L-(LD)BETA*z);
    volatile double lo=(double)((LD)B0+(LD)K*((LD)W-1.0L));
    volatile double coarse=(double)((LD)x*hi);       /* double rounding via 80-bit */
    double r0=(double)((LD)coarse+(LD)x*lo);
    double r1=(double)((LD)x*hi+(LD)x*lo);            /* whole in extended */
    LD loE=(LD)B0+(LD)K*((LD)W-1.0L);
    double r2=(double)((LD)coarse+(LD)x*loE);         /* lo kept extended */
    double r3=(double)((LD)x*hi+(LD)x*loE);
    volatile double c53=x*hi;                        /* pure double coarse */
    double r4=(double)((LD)c53+(LD)x*lo);
    double r5=(double)((LD)c53+(LD)x*loE);
    double r6=x*hi+x*lo;
    double r7=(double)((LD)coarse+(LD)(x*lo));
    double r[8]={r0,r1,r2,r3,r4,r5,r6,r7};
    for(int k=0;k<8;k++) if(r[k]==E[i]) c[k]++;
  }
  const char*nm[8]={"x87 coarse stored, lo dbl","all ext, lo dbl","coarse stored, lo ext","all ext, lo ext","coarse dbl, ext sum lo dbl","coarse dbl, lo ext","pure double","coarse x87, fine dbl prod"};
  for(int k=0;k<8;k++) printf("%-30s %d/%d\n",nm[k],c[k],N);
}
