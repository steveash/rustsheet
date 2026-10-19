# T-1941 W4 run ladders on (1.5,2]: kmyst-gam-q421..q433, one per rung j = 22,24,..,46.
# Each case: 6 runs (a in 1.625,1.75,1.875; s = -1,+1) of 4,096 consecutive doubles from fl(a + s*2^-j).
# Rows: run k occupies rows 4096*k+1 .. 4096*(k+1). B = GAMMALN.PRECISE(A).
import json, math, sys, os
D=sys.argv[1]
for n,j in enumerate(range(22,47,2)):
    p=os.path.join(D,f'kmyst-gam-q{421+n}.case.json'); assert not os.path.exists(p), p
    cells={}; r=1; runs=[]
    for a in (1.625,1.75,1.875):
        for s in (-1,1):
            x=a+s*2.0**-j; runs.append(f'{a}{"+" if s>0 else "-"}2^-{j}')
            for _ in range(4096):
                cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'; r+=1; x=math.nextafter(x,math.inf)
    json.dump({'format':'rustsheet-case/v1','description':f'Kernel-mystery probe (T-1941 W4): GAMMALN.PRECISE (1.5,2] run ladder rung j={j}; 6 runs of 4,096 consecutive doubles from '+', '.join(runs)+'. B = GAMMALN.PRECISE(x).',
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
print('ok')
