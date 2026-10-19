# T-1941 W6: fast law-assignment enumeration in the exact function null space. c(z) = c0 + V z (V: null directions of
# the function rows, from nullspace.Null); a law hypothesis (chain, law T/S/Pn1, j, sign, int, eoff) gives rows linear
# in z (stage derivatives over 2^E, E from c0 + eoff); solve LS + ridge in z (float64), then check self-consistency
# with the solution's own exponents. Usage as module: FastEnum(n, thr).single(track) / .joint(tracks, assigns)
import numpy as np, itertools
from nullspace import Null
from stagelaws import stage_F
def law_key(law,j): return ('S%d'%j if law=='S' else 'T%d'%j) if law in ('T','S') else 'P%dn%s'%(j,law[2:])
class FastEnum:
    def __init__(self,n=9,thr=1e-15,ridge=1e-3,ftol=None,c0=None,d=None):
        # thr: singular-value cut for the free subspace; ftol: if set, add soft function rows sigma_k/ftol * z_k so
        # directions with sigma above ftol are penalised (lets thr be large for far-away truths).
        d=n if d is None else d
        self.ns=Null(n,d,thr=thr); self.n=max(n,d); self.nN=n; self.nD=d; self.k=self.ns.k; self.ridge=ridge
        self.fw=None if ftol is None else self.ns.S[len(self.ns.S)-self.k:]/ftol
        self.c0=self.ns.c0; self.V=self.ns.Vn            # c has length nN+1+nD: N0..NnN, D1..DnD
    def chain(self,z): c=self.c0+z@self.V; return c[:self.nN+1],np.concatenate([[1.0],c[self.nN+1:]])
    def drow(self,ch,j,x):
        """row r (len 2n+1) with r.c = s_j'(x) for chain ch."""
        deg=self.nN if ch=='N' else self.nD
        r=np.zeros(self.nN+1+self.nD); off=0 if ch=='N' else self.nN
        for k in range(j+1,deg+1):
            if ch=='D' and k==0: continue
            r[off+k]+= (k-j)*x**(k-j-1)
        return r
    def rows(self,hyp,x,F,N,D):
        ch,law,j,sg,kk,eo=hyp; A=[]; b=[]
        c=N if ch=='N' else D
        for xv,fo in zip(x,F):
            sf=stage_F(c,np.array([xv]))
            if law in ('T','S'):
                E=np.floor(np.log2(abs(sf['S%d'%j][1][0])))+eo+(1 if law=='T' else 0); r=self.drow(ch,j,xv)/2.0**E
            else:
                EP=np.floor(np.log2(abs(sf['P%d'%j][1][0])))+eo; EM=np.floor(np.log2(abs(sf['S%d'%(j+1)][1][0])))+eo
                r=self.drow(ch,j,xv)/2.0**EP-int(law[2:])*self.drow(ch,j+1,xv)/2.0**EM
            # D0 = 1 contributes nothing to derivatives; r.c0 + (r.V^T) z = sg*fo + kk
            A.append(r@self.V.T); b.append(sg*fo+kk-r@self.c0)
        return np.array(A),np.array(b)
    def solve(self,hyps,tracks):
        N0,D0=self.chain(np.zeros(self.k)); As=[];bs=[]
        for h,(x,F) in zip(hyps,tracks):
            A,b=self.rows(h,x,F,N0,D0); s=1/np.maximum(np.abs(b).max(),1e-300); As.append(A); bs.append(b)
        extra=[self.ridge*np.eye(self.k)]; eb=[np.zeros(self.k)]
        if self.fw is not None: extra.append(np.diag(self.fw)); eb.append(np.zeros(self.k))
        A=np.vstack(As+extra); b=np.concatenate(bs+eb)
        z,*_=np.linalg.lstsq(A,b,rcond=None)
        N,D=self.chain(z); res=[]
        for h,(x,F) in zip(hyps,tracks):
            ch,law,j,sg,kk,eo=h; pr=stage_F(N if ch=='N' else D,np.asarray(x))[law_key(law,j)][0]
            res.append(np.sqrt(np.mean((pr-(sg*np.asarray(F)+kk))**2)))
        return z,res
    def single(self,x,F,laws=('T','Pn1'),eoffs=(0,),top=15):
        N0,D0=self.chain(np.zeros(self.k)); out=[]
        for ch in 'ND':
            deg=self.nN if ch=='N' else self.nD
            for law in laws:
                for j in range(deg-1 if law[0]=='P' else deg):
                    for eo in eoffs:
                        pr=stage_F(N0 if ch=='N' else D0,np.asarray(x))[law_key(law,j)][0]*2.0**(-eo)
                        for sg in (1,-1):
                            k0=int(np.round(np.median(pr-sg*np.asarray(F))))
                            for kk in range(k0-2,k0+3):
                                h=(ch,law,j,sg,kk,eo)
                                try: _,res=self.solve([h],[(x,F)])
                                except Exception: continue
                                out.append((res[0],h))
        out.sort(key=lambda t:t[0]); return out[:top]
def joint_beam(fe,tracks,cands,beam=300,show=6):
    """tracks: list of (x,F); cands: per-track list of hypotheses. Beam over tracks in order, score = worst rms."""
    B=[((),0.0)]
    for ti in range(len(tracks)):
        nb=[]
        for a,_ in B:
            for h in cands[ti]:
                aa=a+(h,)
                try: _,res=fe.solve(list(aa),tracks[:ti+1])
                except Exception: continue
                nb.append((aa,max(res)))
        nb.sort(key=lambda t:t[1]); B=nb[:beam]
        print('  after %d tracks: %s'%(ti+1,' '.join('%.1e'%s for _,s in B[:show])),flush=True)
    return B
def prep(t,npts=10):
    sub=np.linspace(0,len(t['x'])-1,min(npts,len(t['x']))).astype(int)
    return (np.array(t['x'])+32768*2.0**-52)[sub],np.array(t['F'])[sub]
def joint_beam_skip(fe,tracks,cands,beam=200,thr=2e-6):
    """Like joint_beam, but each track may be skipped; partial assignments must keep worst rms < thr; ranked by
    (number of skips, worst rms)."""
    B=[((),0.0,0)]
    for ti in range(len(tracks)):
        nb=[]
        for a,s,ns in B:
            nb.append((a+(None,),s,ns+1))
            for h in cands[ti]:
                aa=a+(h,); act=[(hh,tracks[i]) for i,hh in enumerate(aa) if hh is not None]
                try: _,res=fe.solve([p[0] for p in act],[p[1] for p in act])
                except Exception: continue
                if max(res)<thr: nb.append((aa,max(res),ns))
        nb.sort(key=lambda t:(t[2],t[1])); B=nb[:beam]
        print('  after %d tracks: best skips %d rms %.1e | %s'%(ti+1,B[0][2],B[0][1],' '.join('%d/%.0e'%(b[2],b[1]) for b in B[1:5])),flush=True)
    return B
