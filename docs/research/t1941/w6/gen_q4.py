# T-1941 W6 probes for GAMMALN.PRECISE on (1.5,2]. Usage: gen_q4.py CASES_DIR. Every cell B = GAMMALN.PRECISE(A).
#  q401-q408 sweep: 256 segments x 512 consecutive doubles across (1.5,2), irregular starts (32 segments per case).
#  q409 ladder at 2-: x = 2 - c*2^-j, j = 2..52, 64 irregular c in [1,2) (3,264 rows).
#  q410 ladder at 1.5+: x = 1.5 + c*2^-j, j = 2..52, same c (3,264 rows).
#  q411-q414: 16,384 consecutive doubles from 1.5531, 1.5913, 1.9077, 1.9312 (fill the gaps between existing runs).
import json, math, sys, os
D=sys.argv[1]
def up(x0,n):
    xs=[x0]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],math.inf))
    return xs
def write(k,xs,desc):
    name=f'kmyst-gam-q{k}'; p=os.path.join(D,name+'.case.json')
    assert not os.path.exists(p), p
    cells={}
    for r,x in enumerate(xs,1): cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'
    json.dump({'format':'rustsheet-case/v1','description':f'Kernel-mystery probe (T-1941 W6): GAMMALN.PRECISE on (1.5,2], {desc}. B = GAMMALN.PRECISE(x).',
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
phi=(math.sqrt(5)-1)/2
for c in range(8):
    xs=[]
    for g in range(32*c,32*c+32):
        x0=1.5+(g+0.2+0.6*((g*phi)%1.0))*(0.5/256); xs+=up(x0,512)
    write(401+c,xs,f'sweep segments {32*c}..{32*c+31} of 256, 512 consecutive doubles each')
cs=[1+((i*phi+0.137)%1.0) for i in range(64)]
write(409,[2-c*2.0**-j for j in range(2,53) for c in cs],'ladder x = 2 - c*2^-j, j = 2..52, 64 irregular c')
write(410,[1.5+c*2.0**-j for j in range(2,53) for c in cs],'ladder x = 1.5 + c*2^-j, j = 2..52, 64 irregular c')
for k,x0 in zip((411,412,413,414),(1.5531,1.5913,1.9077,1.9312)): write(k,up(x0,16384),f'16,384 consecutive doubles from x0={x0}')
print('ok')
