# T-1941 W6 long runs for GAMMALN.PRECISE on (1.5,2): 24 cases kmyst-gam-L101..L124, 65,536 consecutive doubles each
# from x0_i = 1.5047 + 0.0203913*i + 0.00071*((i*0.618034) mod 1), i = 0..23. B = GAMMALN.PRECISE(A).
import json, math, sys, os
D=sys.argv[1]
for i in range(24):
    x0=1.5047+0.0203913*i+0.00071*((i*0.618034)%1.0); xs=[x0]
    for _ in range(65535): xs.append(math.nextafter(xs[-1],math.inf))
    p=os.path.join(D,f'kmyst-gam-L{101+i}.case.json'); assert not os.path.exists(p), p
    cells={}
    for r,x in enumerate(xs,1): cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'
    json.dump({'format':'rustsheet-case/v1','description':f'Kernel-mystery probe (T-1941 W6): GAMMALN.PRECISE (1.5,2) long run, 65,536 consecutive doubles from x0={x0!r} (i={i}). B = GAMMALN.PRECISE(x).',
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
print('ok')
