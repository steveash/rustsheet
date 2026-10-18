import json,glob,os,sys
out=open('pts.txt','w'); n=0
for f in sorted(glob.glob('/home/user/rustsheet/eval/suites/probes-numeric/cases/kmyst-erfc-*.case.json')):
    o=f.replace('.case.json','.oracle.json')
    if not os.path.exists(o): continue
    c=json.load(open(f))['sheets'][0]['cells']; v=json.load(open(o))['sheets']['Calc']
    # find columns whose formula is ERFC.PRECISE(A) and EXP(-(A*A)) on same row
    for k,fm in c.items():
        if not isinstance(fm,str) or not fm.startswith('=ERFC.PRECISE(A'): continue
        col=k.rstrip('0123456789'); r=k[len(col):]
        if fm!=f'=ERFC.PRECISE(A{r})': continue
        ek=None
        for cc in 'BCDEFG':
            if c.get(f'{cc}{r}')==f'=EXP(-(A{r}*A{r}))': ek=f'{cc}{r}'
        if not ek: continue
        x=c.get(f'A{r}'); y=v.get(k,{}).get('v'); e=v.get(ek,{}).get('v')
        if not isinstance(x,(int,float)) or not isinstance(y,float) or not isinstance(e,float): continue
        if 4<x<=26.6: out.write(f'{float(x).hex()} {y.hex()} {e.hex()} {os.path.basename(f)[:-10]}\n'); n+=1
print(n)
