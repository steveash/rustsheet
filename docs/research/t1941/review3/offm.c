#include <math.h>
#include <stdlib.h>
typedef long double LD;
static double sd(LD v){volatile double d=(double)v;return d;}
static double shiftu(double v,int a){for(int j=0;j<abs(a);j++) v=nextafter(v,a>0?INFINITY:-INFINITY); return v;}
static double tail(const double*c,int n,int k,double s,double x){for(int i=k+1;i<n;i++){s=sd((LD)s*x);s=sd((LD)s+c[i]);}return s;}
/* hand-off offset meter (T-1941 W2): stored raw-x Horner head up to the stored product P_k; pre-rounding sum v = P_k + c[k] + delta*u
   (u = ulp of the stored sum), delta on the grid (i-G)/R for N and D independently; stored tail; y = fl(t*fl(N/D)).
   counts[iN*(2G+1)+iD] += exact match. Integer shifts of the stored sums are clamped at +-4 ulps.
   build: gcc -O2 -shared -fPIC -o liboffm.so offm.c -lm */
void offm(const double*p,int np,const double*q,int nq,int kN,int kD,const double*x,const double*Y,int N,int G,int R,long long*counts){
  int W=2*G+1, A=4, AW=2*A+1;
  int *aN=malloc(sizeof(int)*W), *aD=malloc(sizeof(int)*W); unsigned char M[9][9];
  for(int k=0;k<N;k++){double X=x[k];
    double s=p[0]; for(int i=1;i<kN;i++){s=sd((LD)s*X);s=sd((LD)s+p[i]);} double PN=sd((LD)s*X); LD vN=(LD)PN+(LD)p[kN]; double bN=sd(vN);
    double d=q[0]; for(int i=1;i<kD;i++){d=sd((LD)d*X);d=sd((LD)d+q[i]);} double PD=sd((LD)d*X); LD vD=(LD)PD+(LD)q[kD]; double bD=sd(vD);
    double uN=nextafter(fabs(bN),INFINITY)-fabs(bN), uD=nextafter(fabs(bD),INFINITY)-fabs(bD);
    for(int i=0;i<W;i++){double sN=sd(vN+(LD)(i-G)/R*uN); int a=(int)lrint((double)(((LD)sN-(LD)bN)/uN)); aN[i]=a<-A?-A:(a>A?A:a);
                         double sD=sd(vD+(LD)(i-G)/R*uD); int b=(int)lrint((double)(((LD)sD-(LD)bD)/uD)); aD[i]=b<-A?-A:(b>A?A:b);}
    double t=sd((LD)X-2.0); double nv[9],dv[9];
    for(int a=-A;a<=A;a++){nv[a+A]=tail(p,np,kN,shiftu(bN,a),X); dv[a+A]=tail(q,nq,kD,shiftu(bD,a),X);}
    for(int a=0;a<AW;a++) for(int b=0;b<AW;b++) M[a][b]=(sd((LD)t*sd((LD)nv[a]/dv[b]))==Y[k]);
    for(int i=0;i<W;i++){unsigned char*row=M[aN[i]+A]; long long*cr=counts+(long)i*W; for(int j=0;j<W;j++) cr[j]+=row[aD[j]+A];}
  }
  free(aN);free(aD);}
