# With fitted (f,parity) rules at s4/d3 (BEST per run), scan a constant pre-rounding offset at each late sum s5..s9, d4..d8, at q=N/D and at y.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def rn(v,g,odd):
    q=v/g; f=np.floor(q); r=q-f; fo=np.mod(f,2)==1
    up=(r>0.5)|((r==0.5)&(fo if not odd else ~fo)); return (f+up)*g
def chain(c,x,offs,hand,K):
    s=LD(c[0])+0*x
    for i in range(1,K+1):
        v=sd(s*x)+LD(c[i])
        if i in offs: v=v+offs[i]*ulp(sd(v))
        if i==K: g=ulp(sd(v)); s=rn(v+hand[0]*g,g,hand[1])
        else: s=sd(v)
    return s
def tailo(c,k,s,x,site=None,off=0.0):
    for i in range(k+1,len(c)):
        v=sd(s*x)+LD(c[i])
        if i==site: v=v+off*ulp(sd(v))
        s=sd(v)
    return s
BEST={'r112':((0.25,-0.25,0.25,True),(-0.25,0.0,True)),'r113':((0.25,0.5,0.0,True),(-0.25,0.125,True)),'r610':((0.0,0.375,0.0,True),(0.25,-0.375,True)),'v401a':((0.25,0.0,0.0,True),(0.5,0.0,False)),'r703':((-0.25,0.125,-0.25,True),(-0.25,0.25,True)),'r901':((-0.25,-0.25,0.25,True),(0.0,-0.25,True))}
OFF=np.arange(-8,9)/8.0
for case,(bn,bd) in BEST.items():
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2))
    s4=chain(P0,X,{2:bn[0],3:bn[1]},(bn[2],bn[3]),4); d3=chain(Q0,X,{2:bd[0]},(bd[1],bd[2]),3)
    def yfull(sN=None,oN=0.0,sD=None,oD=0.0,oq=0.0,oy=0.0):
        N=tailo(P0,4,s4,X,sN,oN); D=tailo(Q0,3,d3,X,sD,oD); q=N/D; q=q+oq*ulp(sd(q)) if oq else q; q=sd(q); y=t*q; y=y+oy*ulp(sd(y)) if oy else y; return sd(y).astype(np.float64)
    out=[f"== {case}: {np.mean(yfull()==Y):.3f}"]
    for site in range(5,10): r=max((np.mean(yfull(sN=site,oN=o)==Y),o) for o in OFF); out.append(f"s{site} {r[0]:.3f}({r[1]:+.3f})")
    for site in range(4,9): r=max((np.mean(yfull(sD=site,oD=o)==Y),o) for o in OFF); out.append(f"d{site} {r[0]:.3f}({r[1]:+.3f})")
    r=max((np.mean(yfull(oq=o)==Y),o) for o in OFF); out.append(f"q {r[0]:.3f}({r[1]:+.3f})"); r=max((np.mean(yfull(oy=o)==Y),o) for o in OFF); out.append(f"y {r[0]:.3f}({r[1]:+.3f})")
    print(" ".join(out), flush=True)
