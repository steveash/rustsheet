#include "common.h"
/* score each coefficient line (P n c.. Q m d..) on a run file */
int main(int argc,char**argv){ FILE*f=fopen(argv[2],"r"); static double X[800000],Y[800000]; int n=0; char a[64],b[64]; int id; while(fscanf(f,"%d %63s %63s",&id,a,b)==3){X[n]=strtod(a,0);Y[n]=strtod(b,0);n++;}
 FILE*cf=fopen(argv[1],"r"); char line[4096]; long all=0,tot=0; while(fgets(line,sizeof line,cf)){ char*p=line; char t[8]; int k,used; for(int w=0;w<2;w++){ sscanf(p,"%7s %d%n",t,&k,&used); p+=used; double*c=w?Qd:Pd; if(w)nq=k; else np=k; for(int i=0;i<=k;i++){ char s[64]; sscanf(p,"%63s%n",s,&used); p+=used; c[i]=strtod(s,0);} }
  int h=0; for(int i=0;i<n;i++){ double x=X[i]; double z=st((LD)x*x); h+=fin(x,horner(np,Pd,z,0),horner(nq,Qd,z,0))==Y[i]; } tot++; all+=(h==n); }
 printf("candidates %ld, perfect on %s (%d pts): %ld\n",tot,argv[2],n,all); }
