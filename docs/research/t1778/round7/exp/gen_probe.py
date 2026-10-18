# Build two EXP probe cases (written only into this directory; the oracle runner copies them if approved).
import json, numpy as np, sys
D='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/inv4/exp-exact'
def rows(fn,cols): 
    return [l.split() for l in open(f'{D}/probe/{fn}')]
def case(name,desc,A):
    cells={}
    for i,a in enumerate(A,1):
        cells[f'A{i}']=a; cells[f'B{i}']=f'=EXP(A{i})'
    c={'format':'rustsheet-case/v1','description':desc,'tags':['M9','probe','numeric','exp'],'expected':{},
       'sheets':[{'name':'Calc','cells':cells}]}
    json.dump(c,open(f'{D}/probe/{name}.case.json','w'))
    print(name,len(A),'rows',2*len(A),'cells')
# case 1: clusters
cl=rows('cluster_raw.txt',0)
A1=[float.fromhex(r[3]) for r in cl]
case('kmyst-erfc-expmap1',
 "EXP bit-exactness probe (T-1778 round 4, exp-exact). A = a, B = EXP(a). Every a is chosen so that the exact 2^t, t = RN64(a*RN64(log2 e)), "
 "lies within 1.5/2048 ulp of a binary64 rounding midpoint, so B reveals the sign of the error of Excel's 64-bit 2^frac(t) (F2XM1) at that point. "
 "Layout: 300 clusters (f0 = target frac(t); 120 in [0,0.585), 180 in [0.585,1)) x 5 exponents n = floor(t) in order (-1,-25,-1000,2,1000) x 20 consecutive-double hits; "
 "row = 1 + 100*cluster + 20*k + j. Within a cluster frac(t) varies by < 6E-9. Generator: inv4/exp-exact/search2.c cluster, gen_probe.py.",A1)
# case 2: spread + split discriminators + repeats
sp=rows('spread_raw.txt',0); sd=rows('split_raw.txt',0)
A2=[float.fromhex(r[0]) for r in sp]+[float.fromhex(r[0]) for r in sd]
rep=[float.fromhex(l.split()[0]) for l in open(f'{D}/probe/repeat_args.txt')]
A2+= rep+rep
case('kmyst-erfc-expmap2',
 f"EXP bit-exactness probe (T-1778 round 4, exp-exact). A = a, B = EXP(a). Rows 1-{len(sp)}: random a (|a| log-uniform in [1/16,708], 30% positive) whose exact 2^t "
 f"(t = RN64(a*RN64(log2 e))) is within 1.5/2048 ulp of a binary64 midpoint. Rows {len(sp)+1}-{len(sp)+len(sd)}: a in (-0.6931,0) where the x87 floor split "
 f"(f = RN64(t+1)) and the exact t predict different results (split discriminator). Rows {len(sp)+len(sd)+1}-{len(A2)}: {len(rep)} previously informative arguments "
 "from kmyst-erfc-* (EXP model mismatches), each repeated twice, to test reproducibility. Generator: inv4/exp-exact/search2.c, gen_probe.py.",A2)
