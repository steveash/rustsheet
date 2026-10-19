import numpy as np, sys
def load(f):
    d={}
    for l in open(f):
        if l.startswith('#') or not l.strip(): continue
        p=l.split(); d[p[0]]=[float.fromhex(v) for v in p[2:]]
    return d
for f,pairs in [('near1_candidate_w1.txt',[('N','D'),('N2','D2'),('N3','D3')]),('base23_candidate_w2.txt',[('N','D')])]:
    d=load(f)
    for a,b in pairs:
        n=d[a]; q=d[b]
        rn=np.roots(n[::-1]); rd=np.roots(q[::-1])
        print(f,a,b,'deg',len(n)-1,len(q)-1)
        print('  N roots',np.sort_complex(rn))
        print('  D roots',np.sort_complex(rd))
        # closest pairs
        for r in rn:
            j=np.argmin(abs(rd-r)); 
            if abs(rd[j]-r)<0.1*max(1,abs(r)): print('  near pair',r,rd[j])
