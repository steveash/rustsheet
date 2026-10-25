"""Structural tie response of R's toms708 bratio: finite difference in y with x fixed (check disabled)."""
import subprocess, math
from ideal_c import EXCEL
def bratio(rows):
    inp="".join(f"{a.hex()} {b.hex()} {x.hex()} {y.hex()}\n" for a,b,x,y in rows)
    out=subprocess.run(["ext/bratio_nochk.bin"],input=inp,capture_output=True,text=True).stdout.split("\n")
    return [float.fromhex(l.split()[0]) for l in out[:len(rows)]]
h=1e-7
for a,b,x,c in EXCEL:
    a,b,x=float(a),float(b),float(x); y=1-x
    w=bratio([(a,b,x,y*(1+h)),(a,b,x,y*(1-h))])
    print(a,b,x,'excel',c,'bratio %+.2f'%((math.log(w[0])-math.log(w[1]))/(2*h)))
