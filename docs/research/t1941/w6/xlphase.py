# T-1941 W6: per-run measured line phases for the lattice, from real (or synthetic) long runs. For each run and each
# law line of the candidate chain: phi = angle(DFT(e_data)/DFT(e_sim))/2pi at the candidate's predicted frequency
# (evaluated at the run centre), with the amplitude ratio |r|. Writes "run x_centre chain law j phi ratio".
# Usage: xlphase.py RFILE CHAIN OUT [prefix]
import sys, collections, re, numpy as np
sys.path.insert(0,sys.path[0])
from lib6 import st, L, horner
from synth import read_chain
from longspec import extractR, resid
import kern
from stagelaws import stage_F
N,D=read_chain(sys.argv[2]); N=np.array(N)/D[0]; D=np.array(D)/D[0]
pref=sys.argv[4] if len(sys.argv)>4 else 'kmyst-gam-L1'
rows=collections.defaultdict(list)
for l in open(sys.argv[1]):
    p=l.split()
    if p[5].startswith(pref): rows[re.sub(r'[a-d]?(:B)?$','',p[5])].append((float.fromhex(p[0]),float.fromhex(p[2]),float.fromhex(p[3])))
lines=[('N','T',j) for j in range(2,8)]+[('D','T',j) for j in range(2,8)]+[('N','Pn1',j) for j in range(0,8)]+[('D','Pn1',j) for j in range(0,8)]
def key(law,j): return ('T%d'%j) if law=='T' else 'P%dn1'%j
def dft(e,f):
    n=len(e); w=np.hanning(n); k=np.arange(n); return np.sum((e-e.mean())*w*np.exp(-2j*np.pi*f*k))
out=open(sys.argv[3],'w')
for rk,v in sorted(rows.items()):
    v=np.array(sorted(v)); x=v[:,0]; n=len(x)
    ed=resid(x,v[:,1],v[:,2])
    R=st(L(horner(list(N[::-1]),x))/L(horner(list(D[::-1]),x))); B=kern.outer(x,R); lo,hi=extractR(x,B); es=resid(x,lo,hi)
    xc=x[n//2]
    for ch,law,j in lines:
        c=N if ch=='N' else D
        try: f=stage_F(c,np.array([xc]))[key(law,j)][0][0]
        except KeyError: continue
        fa=f%1.0; a=dft(ed,fa); b=dft(es,fa)
        if abs(b)<1e-9: continue
        r=a/b; out.write('%s %.17g %s %s %d %.5f %.3f\n'%(rk,xc,ch,law,j,np.angle(r)/(2*np.pi),abs(r)))
    print(rk,flush=True)
