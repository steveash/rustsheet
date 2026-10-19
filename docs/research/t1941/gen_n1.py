# T-1941 W5 K3 coherent-scan data: 128 cases kmyst-gam-n1001..n1128, 16,384 consecutive doubles each from
# x0_i = 0.70 + 0.0060013*(i-1) + 0.00031416 (i = 1..128), B = GAMMALN.PRECISE(A). (W5 asked for q301..q428; renamed to avoid q4xx.)
import json, math, sys, os
D=sys.argv[1]
for i in range(1,129):
    x0=0.70+0.0060013*(i-1)+0.00031416; xs=[x0]
    for _ in range(16383): xs.append(math.nextafter(xs[-1],math.inf))
    p=os.path.join(D,f'kmyst-gam-n{1000+i}.case.json'); assert not os.path.exists(p), p
    cells={}
    for r,x in enumerate(xs,1): cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'
    json.dump({'format':'rustsheet-case/v1','description':f'Kernel-mystery probe (T-1941 W5, K3 coherent scans): GAMMALN.PRECISE near-1 kernel, 16,384 consecutive doubles from x0={x0!r} (i={i}). B = GAMMALN.PRECISE(x).',
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
print('ok')
