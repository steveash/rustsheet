# T-1941 K2 run ladders on (2,3] for the line-law reader: kmyst-gam-q221..q233 (parts a/b), rung j = 22,24,..,46.
# Anchors a = 2.30, 2.55, 2.80, 2.95 (non-dyadic). Each rung: 8 runs of 4,096 consecutive doubles (upward) from fl(a + s*1.37*2^-j),
# order (2.30,-),(2.30,+),(2.55,-),(2.55,+) in part a and (2.80,-),(2.80,+),(2.95,-),(2.95,+) in part b.
# q220a: K=0 reference, 4 runs of 4,096 from fl(a). A = x, B = GAMMALN.PRECISE(A).
import json, math, sys, os
D=sys.argv[1]; AN=(2.30,2.55,2.80,2.95)
def write(name, starts, desc):
    p=os.path.join(D,f'kmyst-gam-{name}.case.json'); assert not os.path.exists(p), p
    cells={}; r=1
    for x in starts:
        for _ in range(4096):
            cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'; r+=1; x=math.nextafter(x,math.inf)
    json.dump({'format':'rustsheet-case/v1','description':'Kernel-mystery probe (T-1941 W2): GAMMALN.PRECISE (2,3] non-dyadic run ladder; '+desc,
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
for n,j in enumerate(range(22,47,2)):
    st=[(a,s,a+s*1.37*2.0**-j) for a in AN for s in (-1,1)]
    for part,sl in (('a',st[:4]),('b',st[4:])):
        write(f'q{221+n}{part}',[x for *_,x in sl],f'rung j={j}, 4 runs of 4,096 consecutive doubles from '+', '.join(f'{a}{"+" if s>0 else "-"}1.37*2^-{j}' for a,s,_ in sl)+'.')
write('q220a',list(AN),'K=0 reference: 4 runs of 4,096 consecutive doubles from fl(a), a = 2.30, 2.55, 2.80, 2.95.')
print('ok')
