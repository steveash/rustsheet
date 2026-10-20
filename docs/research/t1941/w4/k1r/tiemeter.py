import k1lib as K, numpy as np, gsk, sys
ch=sys.argv[1]; k=int(sys.argv[2]); run=sys.argv[3]; truth_delta=float(sys.argv[4]) if len(sys.argv)>4 else None
nm,Y,T=[r for r in K.runs([run])][0]
sim=K.gsimpy.Sim(K.N0,K.D0); n=7
def nudged(d):
    pars=[np.zeros(n) for _ in range(4)]; pars[1 if ch=='N' else 3][k]=d; return sim.G(Y,*pars)
mN=gsk.mask(['mul%d'%(k+1)]) if ch=='N' else 0; mD=gsk.mask(['mul%d'%(k+1)]) if ch=='D' else 0
if truth_delta is not None: T=nudged(truth_delta); print('synthetic truth: AN2 with sum nudge %.3f at %s.S%d'%(truth_delta,ch,k+1))
st=sim.G(Y); va=gsk.G(K.N0,K.D0,mN,mD,Y); diff=st!=va
print('%s y0 %.5f: points where variant differs: %d (%.3f)'%(nm,Y[0],diff.sum(),diff.mean()))
def trip(target):
    fv=np.mean(target[diff]==va[diff]); fs=np.mean(target[diff]==st[diff]); return fv,fs,1-fv-fs
print('target triple (follows variant, stored, neither): %.3f %.3f %.3f'%trip(T))
for d in np.arange(-0.5,0.51,1/16):
    print('  %+.4f  %.3f %.3f %.3f'%((d,)+trip(nudged(d))))
