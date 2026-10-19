# T-1941 second review (Fable 5.1): shared helpers. Exact x87 simulation = numpy longdouble (RN64) then astype(float64)
# (RN53) after every stored op. Row dumps are cached as .npy next to the text dumps (SCR env var, default scratchpad).
import numpy as np, os, sys
L=np.longdouble
HERE=os.path.dirname(os.path.abspath(__file__))
SCR=os.environ.get('SCR','/tmp/claude-0/-home-user-rustsheet/7595dcf3-f672-56fd-b282-3180d039b191/scratchpad')
def st(v): return np.asarray(v,dtype=L).astype(np.float64)
def _cache(txt,cols):
    npy=txt+'.npy'
    if os.path.exists(npy) and os.path.getmtime(npy)>os.path.getmtime(txt): return np.load(npy,allow_pickle=True)
    rows=[]
    for l in open(txt):
        p=l.split(); rows.append(tuple(float.fromhex(p[i]) if p[i]!='nan' else np.nan for i in cols)+(p[-1],))
    arr=np.array(rows,dtype=object); np.save(npy,arr); return arr
def g_rows():
    a=_cache(os.path.join(SCR,'g_rows.txt'),(0,1)); return a[:,0].astype(float),a[:,1].astype(float),a[:,2]
def ln_rows():
    a=_cache(os.path.join(SCR,'ln_rows.txt'),(0,1,2)); return a[:,0].astype(float),a[:,1].astype(float),a[:,2].astype(float),a[:,3]
def load_cand_ND(path):
    N=D=None
    for l in open(path):
        if l.startswith('N '): N=[float.fromhex(v) for v in l.split()[1:]]
        if l.startswith('D '): D=[float.fromhex(v) for v in l.split()[1:]]
    return N,D   # ascending
AN2=os.path.join(HERE,'..','g_candidate_w4_AN2.txt')
def horner_asc(c,y,trace=None,nudge=None):
    """Horner from the top of an ASCENDING list; every op stored. trace: list collecting (P_k,S_k) per stage
    (stage k = 1..n, k=1 is the top product c[-1]*y + c[-2]). nudge=(k,'p'|'s',ulps) shifts one stored value."""
    s=np.full_like(y,c[-1]); k=0
    for ci in c[-2::-1]:
        k+=1
        p=st(L(s)*L(y))
        if nudge and nudge[0]==k and nudge[1]=='p': p=p+nudge[2]*np.spacing(p)
        s=st(L(p)+L(ci))
        if nudge and nudge[0]==k and nudge[1]=='s': s=s+nudge[2]*np.spacing(s)
        if trace is not None: trace.append((p,s))
    return s
def G_AN2(N,D,y,nN=None,nD=None):
    n=horner_asc(N,y,nudge=nN); d=horner_asc(D,y,nudge=nD); return st(L(n)/L(d))
