# T-1941 W6: binade crossings of a chain's stored sums and products on [1.5,2] (sign or exponent changes), and the R
# staircase steps of a data set (jumps in the per-segment mean bias); used to discriminate chain hypotheses.
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from stagelaws import stage_F
def chain_crossings(N,D,lo=1.5,hi=2.0,n=20001):
    x=np.linspace(lo,hi,n); out=[]
    for ch,c in (('N',N),('D',D)):
        for k,(F,v,vd) in stage_F(np.asarray(c,float),x).items():
            if k[0] not in 'SP' or len(k)>2: continue
            e=np.floor(np.log2(np.abs(v)))
            for i in np.where(np.diff(e)!=0)[0]: out.append((x[i],ch+'.'+k))
    return sorted(out)
def stair_steps(stairfile,jump=0.25):
    d=np.loadtxt(stairfile); d=d[d[:,1]==512]; d=d[np.argsort(d[:,0])]; out=[]
    for i in range(1,len(d)):
        if abs(d[i,2]-d[i-1,2])>jump and abs(d[i,2]-d[i-1,2])>3*np.hypot(d[i,3],d[i-1,3]): out.append(((d[i,0]+d[i-1,0])/2,d[i,2]-d[i-1,2]))
    return out
if __name__=='__main__':
    from synth import read_chain
    st=stair_steps(sys.argv[1]); print('steps:',' '.join('%.4f(%+.2f)'%s for s in st))
    for f in sys.argv[2:]:
        N,D=read_chain(f); cr=chain_crossings(np.array(N)/D[0],np.array(D)/D[0])
        hits=sum(1 for xs,_ in st if any(abs(xs-xc)<0.003 for xc,_ in cr))
        print('%s: %d crossings; steps explained %d/%d'%(f.split('/')[-1],len(cr),hits,len(st)))
