#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
typedef long double LD;
typedef struct { double x, y; int f, r; } Pt;
static double st(LD v) { volatile double d = (double)v; return d; }
static Pt *P_; static long NP;
static void loadpts(void){ FILE*f=fopen("pts.bin","rb"); fseek(f,0,SEEK_END); NP=ftell(f)/sizeof(Pt); rewind(f); P_=malloc(NP*sizeof(Pt)); if(fread(P_,sizeof(Pt),NP,f)!=(size_t)NP) exit(1); fclose(f);}
static double Pd[8],Qd[8]; static int np=4,nq=5;
static void loadcoef(const char*fn){ FILE*cf=fopen(fn,"r"); char tag[8]; while(fscanf(cf,"%7s",tag)==1){int n; if(fscanf(cf,"%d",&n)!=1)exit(1); double*c=tag[0]=='P'?Pd:Qd; if(tag[0]=='P')np=n; else nq=n; for(int i=0;i<=n;i++){char s[80]; if(fscanf(cf,"%79s",s)!=1)exit(1); c[i]=strtod(s,0);} } fclose(cf);}
static void savecoef(const char*fn){ FILE*f=fopen(fn,"w"); fprintf(f,"P %d",np); for(int i=0;i<=np;i++) fprintf(f," %a",Pd[i]); fprintf(f,"\nQ %d",nq); for(int i=0;i<=nq;i++) fprintf(f," %a",Qd[i]); fprintf(f,"\n"); fclose(f);}
/* mode 0300 Horner; returns final double; *pre = 80-bit value before last store */
static double horner(int n,const double*c,double z,LD*pre){ double s=c[n]; LD u=s; for(int i=n-1;i>=0;i--){ volatile LD t=(LD)s*(LD)z; double tt=st(t); volatile LD uu=(LD)tt+(LD)c[i]; u=uu; s=st(uu);} if(pre)*pre=u; return s; }
static double fin(double x,double N,double W){ volatile LD p=(LD)x*(LD)N; double co=st(p); volatile LD q=(LD)co/(LD)W; return st(q); }
