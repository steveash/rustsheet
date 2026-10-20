# T-1941 W5 K3 top-stage data: kmyst-gam-k3t001..k3t300 (one run of 16,384 consecutive doubles each, start in [1.2, 1.47))
# and kmyst-gam-k3t301..k3t450 (start in [0.7, 1.47)). Starts quasi-random: u = frac(0.5 + 0.6180339887*i).
# B = GAMMALN.PRECISE(A), C = LN(A).
import json, math, sys, os
D=sys.argv[1]
for i in range(450):
    u=(0.5+0.6180339887*i)%1.0
    x=1.2+0.27*u if i<300 else 0.7+0.77*u
    p=os.path.join(D,f'kmyst-gam-k3t{i+1:03d}.case.json'); assert not os.path.exists(p), p
    cells={}; x0=x
    for r in range(1,16385):
        cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'; cells[f'C{r}']=f'=LN(A{r})'; x=math.nextafter(x,math.inf)
    json.dump({'format':'rustsheet-case/v1','description':f'Kernel-mystery probe (T-1941 W5): K3 top-stage data, 16,384 consecutive doubles from {x0!r}. B = GAMMALN.PRECISE(x), C = LN(x).',
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
print('ok')
