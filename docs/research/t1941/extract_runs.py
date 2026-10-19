import json,glob,os,sys
# dump kmyst-gam rows: x B(gammaln) C(ln) D(gamma) case ; non-numeric -> nan
out=open(sys.argv[1],'w')
for f in sorted(glob.glob('eval/suites/probes-numeric/cases/kmyst-gam*.case.json')):
    o=f.replace('.case.json','.oracle.json')
    if not os.path.exists(o): continue
    c=json.load(open(f))['sheets'][0]['cells']; v=json.load(open(o))['sheets']['Calc']
    r=1; nm=os.path.basename(f)[:-10]
    while f"A{r}" in c:
        def g(k):
            y=v.get(f"{k}{r}",{}).get('v'); return float(y).hex() if isinstance(y,(int,float)) else 'nan'
        out.write(f"{float(c[f'A{r}']).hex()} {g('B')} {g('C')} {g('D')} {nm}\n"); r+=1
