import numpy as np, sys, ctypes, collections, math, os
sys.path.insert(0,'../wt/docs/research/t1941/w6'); from synth import read_chain
l=ctypes.CDLL('./libgsimd2.so'); P=ctypes.POINTER(ctypes.c_double)
l.gcountd2.argtypes=[ctypes.c_int,P,P,ctypes.c_int,ctypes.c_int,ctypes.c_long,P,P,P]; l.gcountd2.restype=ctypes.c_long
p=lambda a: np.ascontiguousarray(a,dtype=np.float64).ctypes.data_as(P)
N,D=read_chain('plain1.txt'); N=np.array(N); D=np.array(D); CH=sys.argv[1]; J=int(sys.argv[2]); ch=0 if CH=='N' else 1; C=N if CH=='N' else D
runs=collections.defaultdict(list)
for li in open('k1R.txt'):
    q=li.split(); runs[q[5]].append((float.fromhex(q[0]),float.fromhex(q[1])))
AG=np.arange(-int(os.environ.get('AM','64')),int(os.environ.get('AM','64'))+1)/16; BG=np.arange(-16,17)/4
for k,v in sorted(runs.items(),key=lambda kv: kv[1][0][0]):
    v=np.array(sorted(set(v))); x=v[:,0]; t=v[:,1]; cons=np.nextafter(x[:-1],np.inf)==x[1:]; br=np.where(~cons)[0]+1
    s=max(np.split(np.arange(len(x)),br),key=len)
    if len(s)<4096: continue
    X=np.ascontiguousarray(x[s][:8192]); T=np.ascontiguousarray(t[s][:8192]); n=len(X)
    sv=np.full(n,C[-1])
    for j in range(6,J-1,-1):
        pp=(sv.astype(np.longdouble)*X).astype(float); pre=pp.astype(np.longdouble)+np.longdouble(C[j]); sv=pre.astype(float)
    u=np.spacing(np.abs(sv)); ramp=np.linspace(-0.5,0.5,n)
    base=l.gcountd2(7,p(N),p(D),ch,J,n,p(X),p(np.zeros(n)),p(T))/n; best=(base,0,0)
    for a in AG:
        for b in BG:
            dl=(a+b*ramp)*u; s_=l.gcountd2(7,p(N),p(D),ch,J,n,p(X),p(dl),p(T))/n
            if s_>best[0]: best=(s_,a,b)
    print('%s y %.5f n %d base %.3f best %.3f at alpha %+.3f beta %+.2f (ulps of %sS%d)'%(k,X[0],n,base,best[0],best[1],best[2],CH,J),flush=True)
