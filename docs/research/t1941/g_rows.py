# T-1941: dump (x, GAMMA(x)) rows for 1 <= x < 2 from kmyst-gam cases, keeping only cells whose formula is exactly
# =GAMMA(A<row>) (column D in other cases may hold other expressions). Output: x_hex g_hex case
import json,glob,os,sys
out=open(sys.argv[1],'w'); n=0
for f in sorted(glob.glob('eval/suites/probes-numeric/cases/kmyst-gam*.case.json')):
    o=f.replace('.case.json','.oracle.json')
    if not os.path.exists(o): continue
    c=json.load(open(f))['sheets'][0]['cells']; v=json.load(open(o))['sheets']['Calc']; nm=os.path.basename(f)[:-10]
    for k,form in c.items():
        if not isinstance(form,str) or not form.startswith('=GAMMA(A'): continue
        r=k[1:]
        if form!=f'=GAMMA(A{r})': continue
        x=float(c[f'A{r}']); y=v.get(k,{}).get('v')
        if isinstance(y,(int,float)) and 1<=x<2: out.write(f"{x.hex()} {float(y).hex()} {nm}\n"); n+=1
print(n,'rows')
