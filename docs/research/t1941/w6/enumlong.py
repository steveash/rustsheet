# T-1941 W6: assignment enumeration for one precise long-run track: for each (chain, j, law in T/S/Pn1/Pn2, sign, int)
# solve function + ridge + this track (per-point exponents from the function-only chain) and rank by track rms.
# Precise tracks (rms ~3e-7 when right) should separate true from wrong assignments.
import sys, os, json, numpy as np
sys.path.insert(0,sys.path[0]); from assemble import solve; from stagelaws import stage_F; from lawrows import rows_for, predict as _pred
T=json.load(open(sys.argv[1])); ti=int(sys.argv[2]); n=int(sys.argv[3]); ridge=float(sys.argv[4])
eoffs=[int(v) for v in (sys.argv[6].split(',') if len(sys.argv)>6 else ['0'])]
t=T[ti]; sub=np.linspace(0,len(t['x'])-1,min(10,len(t['x']))).astype(int); x=(np.array(t['x'])+32768*2.0**-52)[sub]; Fo=np.array(t['F'])[sub]
N0,D0,_,_=solve(n,n,[],iters=1,K=40); c0=N0+D0[1:]
out=[]
for ch,c in (('N',np.array(N0)),('D',np.array(D0))):
    for law in (sys.argv[7].split(',') if len(sys.argv)>7 else ('T','S','Pn1','Pn2')):
        for j in range(n-1 if law.startswith('P') else n):
            F=_pred(N0,D0,ch,law,j,x)
            for sg in (1,-1):
                k0=int(np.round(np.median(F-sg*Fo)))
                for eo in eoffs:
                  # with an exponent offset the predicted frequency scales by 2^-eo: re-centre the integer
                  k1=int(np.round(np.median(F*2.0**(-eo)-sg*Fo)))
                  for kk in range(k1-2,k1+3):
                    try: N1,D1,_,_=solve(n,n,rows_for(ch,c,law,j,sg,kk,x,Fo,eoff=eo),iters=1,ridge=ridge,c0=c0,K=40)
                    except Exception: continue
                    r=np.sqrt(np.mean((_pred(N1,D1,ch,law,j,x)-(sg*Fo+kk))**2))
                    out.append((r,'%s.%s%d sign %+d int %+d eoff %+d'%(ch,law,j,sg,kk,eo),(ch,law,j,sg,kk,eo)))
out.sort()
for r in out[:8]: print('rms %.2e  %s'%r[:2])
if len(sys.argv)>5: json.dump([r[2] for r in out[:int(os.environ.get('NCAND','12'))]],open(sys.argv[5],'w'))
