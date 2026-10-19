import numpy as np
def load_runs(path='../rows.txt'):
    """Split extract_runs.py rows into runs of consecutive doubles; key = case:index."""
    rows={}
    for l in open(path):
        xh,bh,ch,dh,nm=l.split()
        rows.setdefault(nm,[]).append((float.fromhex(xh),float.fromhex(bh),float.fromhex(ch)))
    runs={}
    for nm,v in rows.items():
        a=np.array(v); x=a[:,0]
        br=np.where(np.nextafter(x[:-1],np.inf)!=x[1:])[0]+1
        for i,seg in enumerate(np.split(np.arange(len(x)),br)):
            runs[f"{nm[10:]}:{i}"]=a[seg]
    return runs
