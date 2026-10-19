# T-1941 W6: single-track assignment enumeration. For one track, try every (chain, stage j, law S/T/P, sign, int) with
# the exponent E set self-consistently from the function-only chain (+-1), solve function + ridge + this track, and
# rank hypotheses by track rms (noise floor ~5e-4) and by the shift of the solution from the function-only chain.
import sys, json, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from assemble import solve; from stagelaws import stage_F
T=json.load(open(sys.argv[1])); ti=int(sys.argv[2]); n=int(sys.argv[3]); ridge=float(sys.argv[4])
t=T[ti]; x=np.array(t['x']); Fo=np.array(t['F'])
N0,D0,_,_=solve(n,n,[],iters=1); c0=N0+D0[1:]
out=[]
for ch,c in (('N',np.array(N0)),('D',np.array(D0))):
    SF=stage_F(c,x)
    for law in ('S','T','P'):
        for j in range(n):
            F,v,vd=SF[law+str(j)]
            for sg in (1,-1):
                dl=F-sg*Fo; k=int(np.round(np.median(dl)))
                for kk in (k-1,k,k+1):
                    E=int(np.floor(np.log2(abs(np.median(v)))))
                    tr={'chain':ch,'j':j,'law':law,'sign':sg,'int':kk,'x':t['x'],'F':t['F']}
                    try: N1,D1,res,_=solve(n,n,[tr],E0=[E],iters=1,ridge=ridge,c0=c0)
                    except Exception: continue
                    shift=np.max(np.abs(np.array(N1+D1[1:])/np.array(c0)-1))
                    out.append((res[0],shift,'%s.%s%d sign %+d int %+d E %d'%(ch,law,j,sg,kk,E)))
out.sort()
for r in out[:12]: print('rms %.2e shift %.1e  %s'%r)
