# T-1941 review-2 probes (HANDOFF 11.5): V1 v101-v124, V2 v201-v212, V3 v301-v318, V4 v401-v409.
import json, math, sys, os
D=sys.argv[1]
def run(x,n):
    out=[]
    for _ in range(n): out.append(x); x=math.nextafter(x,math.inf)
    return out
def write(name,xs,cols,desc):
    p=os.path.join(D,f'kmyst-gam-{name}.case.json'); assert not os.path.exists(p), p
    cells={}
    for r,x in enumerate(xs,1):
        cells[f'A{r}']=x
        for c,f in cols: cells[f'{c}{r}']=f'={f}(A{r})'
    json.dump({'format':'rustsheet-case/v1','description':'Kernel-mystery probe (T-1941 review 2, HANDOFF 11.5): '+desc,
               'tags':['M9','probe','numeric','gamma'],'expected':{},'sheets':[{'name':'Calc','cells':cells}]},open(p,'w'),indent=1)
def rungs(prefix,start,anchors,cols,fn):
    n=start
    for a in anchors:
        runs=[(j,run(a+1.37*2.0**-j,8192)) for j in range(46,23,-2)]
        for c in range(3):
            part=runs[4*c:4*c+4]
            write(f'{prefix}{n:02d}',[x for _,r in part for x in r],cols,
                  f'{fn} rungs at non-dyadic anchor a={a}: 4 runs of 8,192 consecutive doubles from a+1.37*2^-j, j='+','.join(str(j) for j,_ in part)+'.')
            n+=1
G=[('D','GAMMA')]; L=[('B','GAMMALN.PRECISE'),('C','LN')]
rungs('v1',1,[1.0731,1.2347,1.3912,1.4683,1.5577,1.6341,1.7718,1.9253],G,'V1 K1 GAMMA (column D)')
for i,y0 in enumerate([1.7050,1.7230,1.7580,1.7690,1.7920,1.8010,1.9320,1.9450,1.9550,1.9680,1.9800,1.9920],1):
    write(f'v2{i:02d}',run(y0,16384)+run(y0+0.0008,16384),G,f'V2 K1 GAMMA unwrap pair: 16,384 consecutive doubles from {y0} then 16,384 from {y0}+0.0008 (column D).')
rungs('v3',1,[0.7413,0.8127,0.9318,1.1731,1.2869,1.4137],L,'V3 K3 GAMMALN.PRECISE (B) and LN (C)')
rungs('v4',1,[2.8731,2.9417,2.9873],L,'V4 K2 GAMMALN.PRECISE (B) and LN (C)')
print('ok')
