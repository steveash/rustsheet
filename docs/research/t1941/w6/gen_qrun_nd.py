# T-1941 non-dyadic run ladders on (1.5,2]: kmyst-gam-q441..q453 (parts a/b), one per rung j = 22,24,..,46.
# Each rung: 6 runs (a in 1.6341, 1.7718, 1.9253; s = -1,+1) of 4,096 consecutive doubles (upward) from fl(a + s*1.37*2^-j),
# plus case q440a/b with the 3 anchor runs from fl(a) itself (K = 0 reference) and a spare.
# Part a = runs (a1,-),(a1,+),(a2,-); part b = (a2,+),(a3,-),(a3,+). A = x, B = GAMMALN.PRECISE(A).
import json, math, sys, os
D=sys.argv[1]; AN=(1.6341,1.7718,1.9253)
def write(name, starts, desc):
    p=os.path.join(D,f'kmyst-gam-{name}.case.json'); assert not os.path.exists(p), p
    cells={}; r=1
    for x in starts:
        for _ in range(4096):
            cells[f'A{r}']=x; cells[f'B{r}']=f'=GAMMALN.PRECISE(A{r})'; r+=1; x=math.nextafter(x,math.inf)
    json.dump({'format':'rustsheet-case/v1','description':'Kernel-mystery probe (T-1941 W6): GAMMALN.PRECISE (1.5,2] non-dyadic run ladder; '+desc,
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
for n,j in enumerate(range(22,47,2)):
    st=[(a,s,a+s*1.37*2.0**-j) for a in AN for s in (-1,1)]
    for part,sl in (('a',st[:3]),('b',st[3:])):
        write(f'q{441+n}{part}',[x for *_,x in sl],f'rung j={j}, 3 runs of 4,096 consecutive doubles from '+', '.join(f'{a}{"+" if s>0 else "-"}1.37*2^-{j}' for a,s,_ in sl)+'.')
write('q440a',list(AN),'K=0 reference: 3 runs of 4,096 consecutive doubles from fl(a), a = 1.6341, 1.7718, 1.9253.')
print('ok')
