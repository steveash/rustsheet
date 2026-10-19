# T-1941 review (Fable 5.1): exact x87 simulators for K2 (grid_a) and K3 (c5_f99t), and the GAMMALN row loader.
import numpy as np
L=np.longdouble
def st(v): return np.asarray(v,dtype=L).astype(np.float64)
def horner_top(c_desc,x):   # c_desc: highest degree first
    s=np.full_like(x,c_desc[0])
    for ci in c_desc[1:]: s=st(L(st(L(s)*L(x)))+L(ci))
    return s
def load_rows_B(lo,hi):
    X=[];B=[];Cc=[];Cs=[]
    for l in open('ln_rows.txt'):
        p=l.split(); x=float.fromhex(p[0])
        if lo<=x<hi and x!=1.0: X.append(x); B.append(float.fromhex(p[1])); Cc.append(float.fromhex(p[2]) if p[2]!='nan' else np.nan); Cs.append(p[3])
    return np.array(X),np.array(B),np.array(Cc),np.array(Cs)
# K3 c5_f99t ascending
N3=[0.0]+[float.fromhex(v) for v in "-0x1.59f97cb2595b3p-11 -0x1.92587e5cc6b87p-6 -0x1.8558f6ae39a6dp-3 -0x1.d3f13713066a8p-2 -0x1.4c00f8658473ap-2 0x1.59e7ca69322a9p-5 0x1.6d0d491d31729p-4 0x1.3038d89b42949p-6 0x1.c3aa7b639fa87p-11 0x1.337ad32ea909fp-19".split()]
D3=[0.0]+[float.fromhex(v) for v in "0x1p-13 0x1.f168c3e77cdcfp-8 0x1.72eb273a4ee66p-4 0x1.6b0b4a0843b69p-2 0x1.18e8390e5354fp-1 0x1.6e6b1d23cccc3p-2 0x1.8a4a651d4508dp-4 0x1.35398ecbcfb5dp-7 0x1.ebfe99899feccp-13 -0x1.4461de6b330d3p-31".split()]
N3=N3[1:]; D3=D3[1:]   # the leading 0 in the file is a column label
def k3(x,N=N3,D=D3):
    A=st(L(horner_top(N[::-1],x))/L(horner_top(D[::-1],x))); t=st(L(x)-L(1.0)); return st(L(t)*L(A)),A
# K2 grid_a, highest degree first (HANDOFF section 3)
P2=[float.fromhex(v) for v in "0x1.5717994ef45d1p-51 0x1.5e5b73810edd1p-16 0x1.0ee2f118ad254p-9 0x1.572bf8683b589p-5 0x1.09fd1ad1da445p-2 0x1.f0a8c05397a9fp-2 -0x1.08305b47998b9p-6 -0x1.0ae34eb3cbad6p-1 -0x1.de852aee8c153p-3 -0x1.22260ed97843dp-6".split()]
Q2=[float.fromhex(v) for v in "0x1.03cadf2db0d9dp-18 0x1.52fc9914ab713p-11 0x1.5303658ef66c8p-6 0x1.bb43b464d3259p-3 0x1.c515177eb1755p-1 0x1.7b39fc9194438p+0 0x1.eddf2040d2bc2p-1 0x1.9b252b2fab347p-3 0x1.00000000000eap-7".split()]
def k2(x,P=P2,Q=Q2):
    S=st(L(horner_top(P,x))/L(horner_top(Q,x))); m=st(L(x)-L(2.0)); return st(L(m)*L(S)),S
