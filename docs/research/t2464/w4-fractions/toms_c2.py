import subprocess, math
from ideal_c import EXCEL
def bratio(rows, v):
    inp="".join(f"{a.hex()} {b.hex()} {x.hex()} {y.hex()}\n" for a,b,x,y in rows)
    out=subprocess.run(["ext/bratio_logv.bin",str(v)],input=inp,capture_output=True,text=True).stdout.split("\n")
    return [float.fromhex(l.split()[0]) for l in out[:len(rows)]]
h=1e-7
print("LOGV: 0 TOMS, 1 w1-M1 (lnx=log x; lny=log1p(-x) if x<=.375 else log y), 2 log x, log y, 3 no-y, 4 lnx=log1p(-y)")
for a,b,x,c in EXCEL:
    a,b,x=float(a),float(b),float(x); y=1-x
    out=[]
    for v in range(5):
        w=bratio([(a,b,x,y*(1+h)),(a,b,x,y*(1-h))],v); out.append((math.log(w[0])-math.log(w[1]))/(2*h))
    print(a,b,x,'excel %+.2f'%c,' '.join('%+.2f'%o for o in out))
