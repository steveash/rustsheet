# T-1941 W2 K2 dense runs on (2,3]: kmyst-gam-q201..q216, 32,768 rows each (64 runs x 512 consecutive doubles).
# Run k = 0..1023 (case q2NN holds k = 64*(NN-1)..64*NN-1): x_k = 2 + (k + 0.5 + 0.2357*frac(k*0.618034))/1024.
# B = GAMMALN.PRECISE(A).
import json, math, sys, os
D=sys.argv[1]
for nn in range(1,17):
    p=os.path.join(D,f'kmyst-gam-q2{nn:02d}.case.json'); assert not os.path.exists(p), p
    cells={}; r=1
    for k in range(64*(nn-1),64*nn):
        x=2+(k+0.5+0.2357*((k*0.618034)%1.0))/1024
        for _ in range(512):
            cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'; r+=1; x=math.nextafter(x,math.inf)
    json.dump({'format':'rustsheet-case/v1','description':f'Kernel-mystery probe (T-1941 W2): GAMMALN.PRECISE (2,3] dense runs k={64*(nn-1)}..{64*nn-1}; run k = 512 consecutive doubles from 2+(k+0.5+0.2357*frac(0.618034k))/1024. B = GAMMALN.PRECISE(x).',
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
print('ok')
