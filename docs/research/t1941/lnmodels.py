import sys,math,mpmath
from rnd import *
L2=rn64(mpmath.log(2))
rows=[l.split() for l in open(sys.argv[1])]
models={
 'dr': lambda x: rn53(rn64(mpmath.log(x))),
 'fyl2x(ln2_64*log2x)': lambda x: rn53(rn64(L2*mpmath.log(x,2))),
 'fyl2x then exact': lambda x: rn53(L2*mpmath.log(x,2)),
}
for nm,f in models.items():
    ok=0;miss=[]
    for _,xh,yh,_ in rows:
        x=float.fromhex(xh); y=float.fromhex(yh)
        if f(mpmath.mpf(x))==y: ok+=1
        else: miss.append(x)
    print(f"{nm:24s} {ok}/{len(rows)} misses: {[m for m in miss[:6]]}")
