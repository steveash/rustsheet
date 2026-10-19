# T-1941 W6 interior-anchor ladders for GAMMALN.PRECISE on (1.5,2): x = a +- c*2^-j, j = 6..52, 32 irregular c in [1,2).
# q415/q416: a = 1.625 (+ / -), q417/q418: a = 1.75, q419/q420: a = 1.875. Usage: gen_q4b.py CASES_DIR
import json, math, sys, os
D=sys.argv[1]; phi=(math.sqrt(5)-1)/2
cs=[1+((i*phi+0.311)%1.0) for i in range(32)]
k=415
for a in (1.625,1.75,1.875):
    for sgn,lab in ((1,'+'),(-1,'-')):
        xs=[a+sgn*c*2.0**-j for j in range(6,53) for c in cs]
        name=f'kmyst-gam-q{k}'; p=os.path.join(D,name+'.case.json'); assert not os.path.exists(p), p
        cells={}
        for r,x in enumerate(xs,1): cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'
        json.dump({'format':'rustsheet-case/v1','description':f'Kernel-mystery probe (T-1941 W6): GAMMALN.PRECISE (1.5,2] interior ladder x = {a} {lab} c*2^-j, j = 6..52, 32 irregular c in [1,2) (c_i = 1 + frac(i*0.618034 + 0.311)). B = GAMMALN.PRECISE(x).',
                   'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
        k+=1
print('ok')
