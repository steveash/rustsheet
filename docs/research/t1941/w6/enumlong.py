# T-1941 W6: assignment enumeration for one precise long-run track: for each (chain, j, law in T/S/Pn1/Pn2, sign, int)
# solve function + ridge + this track (per-point exponents from the function-only chain) and rank by track rms.
# Precise tracks (rms ~3e-7 when right) should separate true from wrong assignments.
import sys, json, numpy as np
sys.path.insert(0,sys.path[0]); from assemble import solve; from stagelaws import stage_F
T=json.load(open(sys.argv[1])); ti=int(sys.argv[2]); n=int(sys.argv[3]); ridge=float(sys.argv[4])
t=T[ti]; x=np.array(t['x'])+32768*2.0**-52; Fo=np.array(t['F'])
N0,D0,_,_=solve(n,n,[],iters=1); c0=N0+D0[1:]
def rows_for(ch,c,law,j,sg,kk):
    rows=[]
    for xv,fo in zip(x,Fo):
        sf=stage_F(c,np.array([xv]))
        if law in ('T','S'):
            E=int(np.floor(np.log2(abs(sf['S%d'%j][1][0])))); terms=[(ch,j,law,1,E)]
        else:
            nn=int(law[2:]); EP=int(np.floor(np.log2(abs(sf['P%d'%j][1][0])))); EM=int(np.floor(np.log2(abs(sf['S%d'%(j+1)][1][0]))))
            terms=[(ch,j,'P',1,EP),(ch,j+1,'S',-nn,EM)]
        rows.append({'kind':'comb','terms':terms,'sign':sg,'int':kk,'x':[xv],'F':[fo],'w':1e7})
    return rows
def predict(N,D,ch,law,j):
    c=np.array(N if ch=='N' else D); sf=stage_F(c,x); key=('S%d'%j if law=='S' else 'T%d'%j) if law in ('T','S') else 'P%dn%s'%(j,law[2:])
    return sf[key][0]
out=[]
for ch,c in (('N',np.array(N0)),('D',np.array(D0))):
    for law in ('T','S','Pn1','Pn2'):
        for j in range(n-1 if law.startswith('P') else n):
            F=predict(N0,D0,ch,law,j)
            for sg in (1,-1):
                k0=int(np.round(np.median(F-sg*Fo)))
                for kk in range(k0-2,k0+3):
                    try: N1,D1,_,_=solve(n,n,rows_for(ch,c,law,j,sg,kk),iters=1,ridge=ridge,c0=c0)
                    except Exception: continue
                    r=np.sqrt(np.mean((predict(N1,D1,ch,law,j)-(sg*Fo+kk))**2))
                    out.append((r,'%s.%s%d sign %+d int %+d'%(ch,law,j,sg,kk)))
out.sort()
for r in out[:8]: print('rms %.2e  %s'%r)
