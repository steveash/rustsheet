import sys,os,numpy as np,gsk,math,itertools,time
sys.path.insert(0,sys.argv[1]); from synth import read_chain
d=np.load(sys.argv[2]); x,t=d['x'],d['t']; S=int(os.environ.get('ST','4')); xs,ts=x[::S],t[::S]
N,D=read_chain(sys.argv[3]); C=np.concatenate([N,D]).astype(float); out=sys.argv[4]
mm=lambda L: sum(3<<(2*k) for k in range(L))
MN=mm(int(os.environ.get('LN','0'))); MD=mm(int(os.environ.get('LDm','0')))
free=[i for i in range(16) if i!=8]; U=np.array([math.ulp(abs(v)) for v in C])
def sc(c,X=xs,T=ts): return gsk.count(c[:8],c[8:],MN,MD,X,T)
cur=sc(C)
for rnd in range(int(os.environ.get('ROUNDS','6'))):
    imp=0
    for ids,ds in [((i,),(a,)) for i in free for a in (-4,-3,-2,-1,1,2,3,4)]+[((i,j),(a,b)) for i,j in itertools.combinations(free,2) for a in (-1,1) for b in (-3,-2,-1,1,2,3)]:
        c=C.copy()
        for i,dd in zip(ids,ds): c[i]+=dd*U[i]
        s=sc(c)
        if s>cur: C,cur=c,s; imp+=1
    print('round',rnd,'%.4f'%(cur/len(xs)),'moves',imp,flush=True)
    if not imp: break
print('full %.4f'%(sc(C,x,t)/len(x)))
open(out,'w').write("N "+" ".join(float(v).hex() for v in C[:8])+"\nD "+" ".join(float(v).hex() for v in C[8:])+"\n")
