# T-1941 W6: dump every GAMMALN.PRECISE cell whose argument is A<r>, A<r>+k or A<r>-k (k integer), as
# "arg_hex value_hex case src" with arg = fl(x +- k) (Excel adds in double). Usage: rows_all.py OUT [lo hi]
import json,glob,os,sys,re
out=open(sys.argv[1],'w'); lo=float(sys.argv[2]) if len(sys.argv)>2 else -1e308; hi=float(sys.argv[3]) if len(sys.argv)>3 else 1e308
pat=re.compile(r'^=GAMMALN\.PRECISE\(A(\d+)(?:([+-])(\d+))?\)$'); n=0
for f in sorted(glob.glob('eval/suites/probes-numeric/cases/kmyst-gam*.case.json')):
    o=f.replace('.case.json','.oracle.json')
    if not os.path.exists(o): continue
    c=json.load(open(f))['sheets'][0]['cells']; v=json.load(open(o))['sheets']['Calc']; nm=os.path.basename(f)[:-10]
    for k,form in c.items():
        m=pat.match(form) if isinstance(form,str) else None
        if not m: continue
        a=c.get('A'+m.group(1))
        if not isinstance(a,(int,float,str)): continue
        try: x=float(a)
        except ValueError: continue
        if m.group(2): x = x+int(m.group(3)) if m.group(2)=='+' else x-int(m.group(3))
        y=v.get(k,{}).get('v')
        if isinstance(y,(int,float)) and lo<=x<hi: out.write(f"{x.hex()} {float(y).hex()} {nm} {k[0]}\n"); n+=1
print(n,'rows')
