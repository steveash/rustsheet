# Collect every (EXP argument a, Excel EXP(a)) pair from the erfc-probes run files.
import json, sys, mpmath as mp
P='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/inv3/erfc-probes'
def z87(x):
    p=mp.mpf(x)*mp.mpf(x)
    with mp.workprec(64): q=+p
    return float(q)
xx=['expcol','branchscan','driftmap','runs1','runs2','runs3','runs4','jumpzoom','stepmap1','stepmap2','stepmicro','cutoff']
mp.mp.prec=200
out=open('expdata.txt','w'); seen={}
for c in xx:
    n=0
    for l in open(f'{P}/kmyst-erfc-{c}.txt'):
        col,run,x,v=l.split()
        if col!='C' or v.startswith('!'): continue
        a=-z87(float.fromhex(x)); E=float.fromhex(v)
        out.write(f'{a.hex()} {E.hex()} {c}\n'); n+=1
    print(c,n)
for c in ['chiruns1','chiruns2','chiruns3']:
    n=0
    for l in open(f'{P}/kmyst-erfc-{c}.txt'):
        col,run,x,v=l.split()
        if col!='D' or v.startswith('!'): continue
        a=-float.fromhex(x); E=float.fromhex(v)
        out.write(f'{a.hex()} {E.hex()} {c}\n'); n+=1
    print(c,n)
