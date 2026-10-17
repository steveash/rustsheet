# T-1778: sweep-level V phases with sweepfit (one hi alias + fractional offset, one V phase, one lo offset per 24-run sweep).
# Usage: python3 sweepphase2.py RUNFILE FIRST_SWEEP END_SWEEP [WINDOW]; needs sweepfit built in the run directory.
import math,subprocess,sys
from concurrent.futures import ThreadPoolExecutor
A=1.1170334248975802;U=math.ulp(A);h=[0,0.14068131264230,0.0448191707,0.00180525,0.00018]
G=2.0**-53; c=[0.4546572672773,-0.0598952645,0.02203]
fn=sys.argv[1]; s0,s1=int(sys.argv[2]),int(sys.argv[3]); W=sys.argv[4] if len(sys.argv)>4 else '2'
runs={}
for l in open(fn):
    p=l.split(); r=int(p[0])
    if r not in runs: x=float.fromhex(p[1]); runs[r]=x*x
def job(s):
    zr=runs[s*24+12]; slope=(c[0]+2*c[1]*zr+3*c[2]*zr*zr)/G
    o=subprocess.run(['./sweepfit',fn,str(s*24),str(s*24+23),'0',W,'500',repr(slope),repr(zr)],capture_output=True,text=True,env={'NF':'10'}).stdout.strip()
    return s,zr,o
with ThreadPoolExecutor(8) as ex:
    for s,zr,o in ex.map(job,range(s0,s1)): print('sw%d %r %s'%(s,zr,o)); sys.stdout.flush()
