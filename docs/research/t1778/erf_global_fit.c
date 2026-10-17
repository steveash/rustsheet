/* T-1778 research: global exact-match fit of the small-x erf model (hi = fl(A + a1 z + a2 z^2 + a3 z^3), lo = B0 + K*(exp(-BETA z) - 1)), MODE 0 = 64-bit SSE2 doubles, MODE 1 = x87 80-bit final sum. Usage: erf_global_fit <runs file "id x_hex excel_hex"> <zmax> <mode>. */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#define MAXP 200000
double X[MAXP], E[MAXP]; int N=0;
typedef long double LD;
double A=1.1170334248975802;
int MODE=0; /* 0 = SSE2 double, 1 = x87 ext final sum + double-rounded coarse */
static double P[6]; /* a1,a2,a3,K,beta,B0 */
int score(const double*p){
  int c=0;
  for(int i=0;i<N;i++){double x=X[i]; double z=x*x;
    double hi=A+z*(p[0]+z*(p[1]+z*p[2]));
    double W=exp(-(p[4]*z));
    double lo=p[5]+p[3]*(W-1.0);
    double r;
    if(MODE==0){ r=x*hi+x*lo; }
    else { volatile double co=(double)((LD)x*hi); r=(double)((LD)co+(LD)x*lo); }
    if(r==E[i]) c++;}
  return c;
}
int main(int argc,char**argv){
  FILE*f=fopen(argv[1],"r"); char a[64],b[64]; int id; double zmax=atof(argv[2]); MODE=atoi(argv[3]);
  while(fscanf(f,"%d %63s %63s",&id,a,b)==3){double x=strtod(a,0); if(x*x<=zmax){X[N]=x; E[N]=strtod(b,0); N++;}}
  double p[6]={0.1406813111,-0.004647269,0.0,1.1366973292490381,0.4546572618,0.011345742197932506};
  double s[6]={1e-10,1e-4,1e-3,1e-9,1e-9,0};
  int best=score(p); printf("N=%d start %d\n",N,best);
  for(int it=0;it<200;it++){ int imp=0;
    for(int k=0;k<6;k++){
      for(int d=-4;d<=4;d++){ if(!d) continue; double q[6]; for(int j=0;j<6;j++) q[j]=p[j];
        if(k==5){ q[5]=p[5]; for(int t=0;t<abs(d);t++) q[5]=nextafter(q[5], d>0?1:-1); if(abs(d)>2) continue; }
        else q[k]=p[k]+d*s[k];
        int v=score(q); if(v>best){best=v; for(int j=0;j<6;j++) p[j]=q[j]; imp=1;} }
    }
    if(!imp){ int small=1; for(int k=0;k<5;k++){ s[k]/=2; if(s[k]>fabs(p[k])*1e-17+1e-30) small=0;} if(small) break; }
    if(it%5==0||!imp) printf("it %d best %d/%d a1=%.15g a2=%.12g a3=%.10g K=%.16g beta=%.16g B0=%.17g\n",it,best,N,p[0],p[1],p[2],p[3],p[4],p[5]);
  }
}
