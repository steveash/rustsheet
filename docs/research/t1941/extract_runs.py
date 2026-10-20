import json,glob,os,re,sys
# dump kmyst-gam rows: x GAMMALN.PRECISE LN GAMMA case ; missing or non-numeric -> nan.
# Columns are chosen by each row's formula, never by letter: some cases put other functions in B
# (kmyst-gam-q501/q502 have B = GAMMA, C/D = GAMMA.DIST), which once contaminated K3 run sets.
FUN={'GAMMALN.PRECISE':0,'LN':1,'GAMMA':2}
out=open(sys.argv[1],'w')
for f in sorted(glob.glob('eval/suites/probes-numeric/cases/kmyst-gam*.case.json')):
    o=f.replace('.case.json','.oracle.json')
    if not os.path.exists(o): continue
    c=json.load(open(f))['sheets'][0]['cells']; v=json.load(open(o))['sheets']['Calc']
    r=1; nm=os.path.basename(f)[:-10]
    while f"A{r}" in c:
        vals=['nan']*3
        for col in 'BCDEFGH':
            m=re.fullmatch(rf"=([A-Z.]+)\(A{r}\)",str(c.get(f"{col}{r}",'')))
            if m and m.group(1) in FUN:
                y=v.get(f"{col}{r}",{}).get('v')
                if isinstance(y,(int,float)): vals[FUN[m.group(1)]]=float(y).hex()
        out.write(f"{float(c[f'A{r}']).hex()} {' '.join(vals)} {nm}\n"); r+=1
