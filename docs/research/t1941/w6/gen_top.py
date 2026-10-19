# T-1941 W6 top-pair data on (1.5,2]: 3,000 runs of 4,096 consecutive doubles (upward), 4 runs per case
# -> kmyst-gam-w6t0001..w6t0750 (16,384 rows each). Run i: u = frac(0.5 + 0.6180339887*i);
# x0 = 1.5 + 0.3*u for i % 3 == 0, else 1.8 + 0.1995*u (two thirds of the runs above 1.8). B = GAMMALN.PRECISE(A).
import json, math, sys, os
D=sys.argv[1]
def x0(i):
    u=(0.5+0.6180339887*i)%1.0
    return 1.5+0.3*u if i%3==0 else 1.8+0.1995*u
for c in range(750):
    p=os.path.join(D,f'kmyst-gam-w6t{c+1:04d}.case.json'); assert not os.path.exists(p), p
    cells={}; r=1; starts=[]
    for i in range(4*c,4*c+4):
        x=x0(i); starts.append(repr(x))
        for _ in range(4096):
            cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'; r+=1; x=math.nextafter(x,math.inf)
    json.dump({'format':'rustsheet-case/v1','description':'Kernel-mystery probe (T-1941 W6): GAMMALN.PRECISE (1.5,2] top-pair data, 4 runs of 4,096 consecutive doubles from '+', '.join(starts)+'.',
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
print('ok')
