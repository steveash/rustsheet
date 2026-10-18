#include "common.h"
/* like novscore but prints distribution of misses */
static double X[800000],Y[800000];
int main(int argc,char**argv){ FILE*f=fopen(argv[2],"r"); int n=0; char a[64],b[64]; int id; while(fscanf(f,"%d %63s %63s",&id,a,b)==3){X[n]=strtod(a,0);Y[n]=strtod(b,0);n++;}
 FILE*cf=fopen(argv[1],"r"); char line[4096]; long hist[8]={0},tot=0; long sum=0; while(fgets(line,sizeof line,cf)){ char*p=line; char t[8]; int k,used; for(int w=0;w<2;w++){ sscanf(p,"%7s %d%n",t,&k,&used); p+=used; double*c=w?Qd:Pd; if(w)nq=k; else np=k; for(int i=0;i<=k;i++){ char s[64]; sscanf(p,"%63s%n",s,&used); p+=used; c[i]=strtod(s,0);} }
  int m=0; for(int i=0;i<n;i++){ double x=X[i]; double z=st((LD)x*x); m+=fin(x,horner(np,Pd,z,0),horner(nq,Qd,z,0))!=Y[i]; } tot++; sum+=m; hist[m>6?7:m]++; }
 printf("%s on %s (%d pts): candidates %ld, misses histogram 0..6,7+:",argv[1],argv[2],n,tot); for(int i=0;i<8;i++) printf(" %ld",hist[i]); printf("  mean %.3f\n",(double)sum/tot); }
