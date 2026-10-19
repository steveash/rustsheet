# T-1941 review (Fable 5.1): dump (x, GAMMALN.PRECISE(x), LN(x)) rows keeping only cells whose formula is exactly =GAMMALN.PRECISE(A<row>) (B) and =LN(A<row>) (C)
import json,glob,os,sys
out=open(sys.argv[1],'w'); n=0
for f in sorted(glob.glob('eval/suites/probes-numeric/cases/kmyst-gam*.case.json')):
    o=f.replace('.case.json','.oracle.json')
    if not os.path.exists(o): continue
    c=json.load(open(f))['sheets'][0]['cells']; v=json.load(open(o))['sheets']['Calc']; nm=os.path.basename(f)[:-10]
    for k,form in c.items():
        if not isinstance(form,str) or not k.startswith('B'): continue
        r=k[1:]
        if form!=f'=GAMMALN.PRECISE(A{r})': continue
        x=float(c[f'A{r}']); y=v.get(k,{}).get('v'); ln=v.get(f'C{r}',{}).get('v') if c.get(f'C{r}')==f'=LN(A{r})' else None
        if isinstance(y,(int,float)): out.write(f"{x.hex()} {float(y).hex()} {float(ln).hex() if isinstance(ln,(int,float)) else 'nan'} {nm}\n"); n+=1
print(n,'rows')
