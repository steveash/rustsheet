typedef long double LD;
static double sd(LD v){volatile double d=(double)v;return d;}
/* synthetic truths: per-chain early scheme m through stage ke, then stored raw-x Horner; csim(p,np,q,nq,kN,kD,mN,mD,x,N,y).
   m: 0 Horner, 2 Estrin, 3 FMA, 6 reverse power-sum, 8 80-bit stored at hand-off, 9 80-bit unstored, 10 products stored/sums
   kept, 11 base with last add unstored, 12 even/odd in x^2, 14 pairwise terms, 15 even/odd odd-first, 16 forward power-sum
   (x4=x2^2), 20 = 11 variant, 22/26/32 = 2/6/12 with final op unstored */
static LD early(const double*c,int ke,double x,int m){
  if(m==0){double s=c[0];for(int i=1;i<=ke;i++){s=sd((LD)s*x);s=sd((LD)s+c[i]);}return s;}
  if(m==2){double x2=sd((LD)x*x); if(ke==4){double a=sd((LD)sd((LD)c[0]*x)+c[1]); double b=sd((LD)sd((LD)c[2]*x)+c[3]); double s=sd((LD)sd((LD)a*x2)+b); return sd((LD)sd((LD)s*x)+c[4]);}
            {double a=sd((LD)sd((LD)c[0]*x)+c[1]); double b=sd((LD)sd((LD)c[2]*x)+c[3]); return sd((LD)sd((LD)a*x2)+b);}}
  if(m==3){double s=c[0]; for(int i=1;i<=ke;i++) s=sd(__builtin_fmal((LD)s,(LD)x,(LD)c[i])); return s;}
  if(m==6){double pw[6]; pw[0]=1; pw[1]=x; pw[2]=sd((LD)x*x); pw[3]=sd((LD)pw[2]*x); pw[4]=sd((LD)pw[3]*x);
    double s=c[ke]; for(int i=ke-1;i>=0;i--) s=sd((LD)s+sd((LD)c[i]*pw[ke-i])); return s;}
  if(m==8||m==9){LD s=c[0];for(int i=1;i<=ke;i++){s=s*(LD)x;s=s+(LD)c[i];} return m==8?(LD)sd(s):s;}
  if(m==10){LD s=c[0];for(int i=1;i<=ke;i++){s=s*(LD)x; s=(LD)sd(s); s=s+(LD)c[i];} return s;}
  if(m==11){double s=c[0];for(int i=1;i<ke;i++){s=sd((LD)s*x);s=sd((LD)s+c[i]);} LD v=(LD)s*(LD)x; v=(LD)sd(v); return v+(LD)c[ke];}
  if(m==12){double x2=sd((LD)x*x);
    if(ke==4){double E=sd((LD)sd((LD)sd((LD)sd((LD)c[0]*x2)+c[2])*x2)+c[4]); double O=sd((LD)sd((LD)c[1]*x2)+c[3]); return sd((LD)E+sd((LD)O*x));}
    {double E=sd((LD)sd((LD)c[1]*x2)+c[3]); double O=sd((LD)sd((LD)c[0]*x2)+c[2]); return sd((LD)sd((LD)O*x)+E);}}
  if(m==14){double pw[6]; pw[0]=1; pw[1]=x; pw[2]=sd((LD)x*x); pw[3]=sd((LD)pw[2]*x); pw[4]=sd((LD)pw[3]*x); double T[5];
    for(int i=0;i<=ke;i++) T[i]=(ke-i)?sd((LD)c[i]*pw[ke-i]):c[i];
    if(ke==4) return sd((LD)sd((LD)sd((LD)T[0]+T[1])+sd((LD)T[2]+T[3]))+T[4]); return sd((LD)sd((LD)T[0]+T[1])+sd((LD)T[2]+T[3]));}
  if(m==15){double x2=sd((LD)x*x);
    if(ke==4){double O=sd((LD)sd((LD)c[1]*x2)+c[3]); double s=sd((LD)sd((LD)c[0]*x2)+c[2]); s=sd((LD)sd((LD)s*x2)+c[4]); return sd((LD)sd((LD)O*x)+s);}
    {double O=sd((LD)sd((LD)c[0]*x2)+c[2]); double E=sd((LD)sd((LD)c[1]*x2)+c[3]); return sd((LD)E+sd((LD)O*x));}}
  if(m==16){double pw[6]; pw[0]=1; pw[1]=x; pw[2]=sd((LD)x*x); pw[3]=sd((LD)pw[2]*x); pw[4]=sd((LD)pw[2]*pw[2]);
    double s=sd((LD)c[0]*pw[ke]); for(int i=1;i<ke;i++) s=sd((LD)s+sd((LD)c[i]*pw[ke-i])); return sd((LD)s+c[ke]);}
  if(m==22){double x2=sd((LD)x*x); if(ke==4){double a=sd((LD)sd((LD)c[0]*x)+c[1]); double b=sd((LD)sd((LD)c[2]*x)+c[3]); double s=sd((LD)sd((LD)a*x2)+b); return (LD)sd((LD)s*x)+(LD)c[4];}
            {double a=sd((LD)sd((LD)c[0]*x)+c[1]); double b=sd((LD)sd((LD)c[2]*x)+c[3]); return (LD)sd((LD)a*x2)+(LD)b;}}
  if(m==26){double pw[6]; pw[0]=1; pw[1]=x; pw[2]=sd((LD)x*x); pw[3]=sd((LD)pw[2]*x); pw[4]=sd((LD)pw[3]*x);
    double s=c[ke]; for(int i=ke-1;i>0;i--) s=sd((LD)s+sd((LD)c[i]*pw[ke-i])); return (LD)s+(LD)sd((LD)c[0]*pw[ke]);}
  if(m==32){double x2=sd((LD)x*x);
    if(ke==4){double E=sd((LD)sd((LD)sd((LD)sd((LD)c[0]*x2)+c[2])*x2)+c[4]); double O=sd((LD)sd((LD)c[1]*x2)+c[3]); return (LD)E+(LD)sd((LD)O*x);}
    {double E=sd((LD)sd((LD)c[1]*x2)+c[3]); double O=sd((LD)sd((LD)c[0]*x2)+c[2]); return (LD)sd((LD)O*x)+(LD)E;}}
  if(m==20){double s=c[0];for(int i=1;i<ke;i++){s=sd((LD)s*x);s=sd((LD)s+c[i]);} return (LD)sd((LD)s*x)+(LD)c[ke];}
  return 0;}
static double chain(const double*c,int n,int ke,double x,int m){LD s=early(c,ke,x,m);
  for(int i=ke+1;i<n;i++){s=(LD)sd(s*(LD)x); s=(LD)sd(s+(LD)c[i]);} return (double)s;}
void csim(const double*p,int np,const double*q,int nq,int kN,int kD,int mN,int mD,const double*x,int N,double*y){
  for(int k=0;k<N;k++){double n=chain(p,np,kN,x[k],mN),d=chain(q,nq,kD,x[k],mD);double t=sd((LD)x[k]-2.0);y[k]=sd((LD)t*sd((LD)n/d));}}
