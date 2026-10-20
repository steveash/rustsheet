import numpy as np, sys, ctypes, collections, math
sys.path.insert(0,'../wt/docs/research/t1941/w6'); from synth import read_chain
l=ctypes.CDLL('./libgsimo2.so'); P=ctypes.POINTER(ctypes.c_double)
l.gcounto2.argtypes=[ctypes.c_int]+[P]*6+[ctypes.c_long,P,P]; l.gcounto2.restype=ctypes.c_long
p=lambda a: np.ascontiguousarray(a,dtype=np.float64).ctypes.data_as(P)
N,D=read_chain('plain1.txt'); N=np.array(N); D=np.array(D)
runs=collections.defaultdict(list)
for li in open('k1R.txt'):
    q=li.split(); v=float.fromhex(q[0])
    if v>=1.8: runs[q[5]].append((v,float.fromhex(q[1])))
sel=sorted(runs.items(),key=lambda kv: kv[1][0][0])[int(sys.argv[1])::int(sys.argv[2])]
# runcapctl.py: instead -> import os, gsk; sel=[kv for kv in sorted(...) if kv[0].split(':')[0] in sys.argv[1].split(',')]; Nt,Dt=read_chain(os.environ['TRUTH'])
G=np.arange(-32,33)/16
for k,v in sel:
    v=np.array(sorted(set(v))); x=v[:,0]; t=v[:,1]; cons=np.nextafter(x[:-1],np.inf)==x[1:]; br=np.where(~cons)[0]+1
    s=max(np.split(np.arange(len(x)),br),key=len)
    if len(s)<4096: continue
    X=np.ascontiguousarray(x[s][:4096]); T=np.ascontiguousarray(t[s][:4096]); n=len(X); y0=X[0]; uu={}
    # runcapctl.py: T=np.ascontiguousarray(gsk.G(Nt,Dt,0,0,X))
    for ch,C in (('N',N),('D',D)):
        sv=C[-1]
        for j in range(6,-1,-1):
            pv=float(np.longdouble(sv)*np.longdouble(y0)); uu[(ch,'P',j)]=math.ulp(abs(pv)); sv=float(np.longdouble(pv)+np.longdouble(C[j])); uu[(ch,'S',j)]=math.ulp(abs(sv))
    A={k_:np.zeros(8) for k_ in ('oN','oD','qN','qD')}
    def sc(): return l.gcounto2(7,p(N),p(D),p(A['oN']),p(A['oD']),p(A['qN']),p(A['qD']),n,p(X),p(T))/n
    cur=sc(); base=cur
    for rnd in range(3):
        for ch in 'ND':
            for kind in 'SP':
                key=('o' if kind=='S' else 'q')+ch
                for j in range(7):
                    if ch=='D' and kind=='S' and j==0: continue
                    old=A[key][j]; best=(cur,old)
                    for g in G:
                        A[key][j]=g*uu[(ch,kind,j)]; s_=sc()
                        if s_>best[0]: best=(s_,A[key][j])
                    A[key][j]=best[1]; cur=best[0]
    print('%s y %.5f base %.3f cap %.3f'%(k,y0,base,cur),flush=True)
