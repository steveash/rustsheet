"""Usage: score_ga.py DIR MODE (x87 or sse). Scores garef.py on DIR/ga.sample by region and, for x87, writes
DIR/ga.fixture.x87 (x excel model as f64 bit patterns)."""
import sys, struct
S=sys.argv[1]; mode=sys.argv[2]
sys.argv=[sys.argv[0], S, mode]
exec(open(S+'/garef.py').read())
rows=[l.split() for l in open(S+'/ga.sample')]
res={}
out=[]
for a,b,r in rows:
    x=float.fromhex(a); y=float.fromhex(b); m=gamma(x)
    t=res.setdefault(r,[0,0]); t[1]+=1; t[0]+= (m==y)
    out.append((x,y,m))
for r,(ok,n) in res.items(): print(mode, r, n, round(ok/n,4))
if len(sys.argv)>3 or mode=="x87":
    bits=lambda f: format(struct.unpack('<Q',struct.pack('<d',f))[0],'016x')
    with open(S+'/ga.fixture.'+mode,'w') as o:
        for x,y,m in out: o.write(f"{bits(x)} {bits(y)} {bits(m)}\n")
