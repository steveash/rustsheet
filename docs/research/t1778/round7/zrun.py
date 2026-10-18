import numpy as np
from chain import *
B2=float.fromhex('0x1.0191f83634b97p+4')
def two_sum(p,q):
    s=p+q; bv=s-p; return s,(p-(s-bv))+(q-bv)
def zruns(prev_fn, cst, data=None):
    """prev_fn(x)->stored product before the add; Xk=fl(prev+cst); Y=fl(Xk*x). per-run normalized Z."""
    x,y,run,cnt=load() if data is None else data
    Pv=prev_fn(x); Xk=Pv+cst; Y,e=tp(Xk,x); uY=ulp(Y); th=e/uY; Am=uY/Y/2**-53
    z=y*Am*np.exp(-2j*np.pi*th)
    out=[]
    for r in np.unique(run):
        m=run==r
        if np.unique(uY[m]).size>1 or np.unique(ulp(Xk[m])).size>1 or np.unique(ulp(Pv[m])).size>1: continue
        i=np.where(m)[0][m.sum()//2]
        zz=z[m]; Z=np.sum(zz)/np.sqrt(np.sum(np.abs(zz)**2))
        out.append((x[i],uY[i],Z,ulp(Pv[i]),ulp(Xk[i]),r))
    a=np.array(out,dtype=object)
    return dict(xc=a[:,0].astype(float),uY=a[:,1].astype(float),Z=a[:,2].astype(complex),uP=a[:,3].astype(float),uX=a[:,4].astype(float),run=a[:,5].astype(int))
def Wfn(x):
    _,_,T=front(x); V=T+B2; return V*x
def cscan(D,sel,deltas,xref):
    ph=(D['xc'][sel]-xref)/D['uY'][sel]; Z=D['Z'][sel]
    S=np.zeros(len(deltas))
    for i in range(0,len(deltas),5000):
        d=deltas[i:i+5000]; S[i:i+5000]=np.abs(np.exp(-2j*np.pi*np.outer(d,ph))@Z)**2
    return S
def jscan(D,sel,g,H,binw=1.0,center=0,top=10):
    """exact scan over j in [center-H,center+H]: candidate shift j*g (g power of 2). Coherent per x-bin."""
    eg=int(np.log2(g)); incs=[]
    for xc,u in zip(D['xc'][sel],D['uY'][sel]):
        mant,ex=np.frexp(xc); X=int(mant*2**53); exx=int(ex)-53; eu=int(np.log2(u))
        K=eu-exx-eg; assert 0<K<=64,K
        incs.append((X<<(64-K))%(2**64))
    incs=np.array(incs,dtype=np.uint64); Z=D['Z'][sel]
    bins=np.floor(D['xc'][sel]/binw).astype(int); ub=np.unique(bins); masks=[bins==b for b in ub]
    best=[]; ch=20000
    for s in range(center-H,center+H+1,ch):
        d=np.arange(s,min(s+ch,center+H+1),dtype=np.int64).astype(np.uint64)
        ph=(np.multiply.outer(d,incs)).astype(np.float64)/2**64
        E=np.exp(-2j*np.pi*ph)*Z
        per=np.array([np.abs(E[:,m].sum(1))**2 for m in masks]); tot=per.sum(0)
        for kk in np.argsort(-tot)[:top]: best.append((tot[kk],int(s+kk) if s+kk<2**62 else int(s+kk),per[:,kk]))
    best.sort(key=lambda t:-t[0])
    return best[:top],ub,sum(np.sum(np.abs(Z[m])**2) for m in masks)
B3=float.fromhex('0x1.d936f5138efd9p+5')
def Yfn(x):
    return (Wfn(x)+B3)*x
B4=float.fromhex('0x1.396f0fc8e5c9cp+7')
def Zfn(x):
    return (Yfn(x)+B4)*x
B5=float.fromhex('0x1.26c409d384172p+8')
def Z5fn(x):
    return (Zfn(x)+B5)*x
B6=float.fromhex('0x1.80fb2b5716d18p+8')
def Z6fn(x):
    return (Z5fn(x)+B6)*x
B7=float.fromhex('0x1.3da266cd039e0p+8')
def Z7fn(x):
    return (Z6fn(x)+B7)*x
BETA=float.fromhex('0x1.27cdbb941913dp-2'); BD=float.fromhex('0x1.4324d3ef0f238p+1')
def DS1x(x):
    return (BETA*x+BD)*x
