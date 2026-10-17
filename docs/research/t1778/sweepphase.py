# T-1778: W rounding phase per sweep (interval method). Usage: python3 sweepphase.py RUNFILE GROUPS d|p
# GROUPS: comma list of 24-run sweep indices or first-last run ranges; d = lo-offset sawtooth, p = per-run phase scan.
# Needs covd and phscan built in the run directory.
import math,subprocess,re,sys
from concurrent.futures import ThreadPoolExecutor
A=1.1170334248975802;U=math.ulp(A);h=[0,0.14068131264230,0.0448191707,0.00180525,0.00018]
K=1.1366973296084131; G=2.0**-53; b=0.45465674353760216
fn=sys.argv[1]; sweeps=[[int(x) for x in g.split('-')] for g in sys.argv[2].split(',')]; mode=sys.argv[3]
runs={}
for l in open(fn):
    p=l.split(); r=int(p[0])
    if r not in runs: x=float.fromhex(p[1]); runs[r]=x*x
def delta_run(r):
    z=runs[r]; pr=sum(h[k]*z**k for k in range(1,5))/U; m0=round(pr); best=None
    for m in (m0-1,m0,m0+1):
        o=subprocess.run(['./covd3',fn,str(r),'5','0']+[repr(v) for v in h[1:]]+[str(m)],capture_output=True,text=True,env={'KW':'0','BETA':repr(b),'WF':'1'}).stdout.split()
        c=int(o[1]); d=(float(o[2]),float(o[3]))
        key=(c,-abs(m-pr))
        if best is None or key>best[3]: best=(c,m,d,key)
    return (z,)+best[:3]
def ph_run(r):
    z=runs[r]; m=round(sum(h[k]*z**k for k in range(1,5))/U)
    W=3 if z<1e-3 else 40
    o=subprocess.run(['./phscan',fn,str(r),str(m),str(W),'500',repr(b)],capture_output=True,text=True).stdout.split()
    n=int(o[4]); best=int(o[6]); ivs=[tuple(map(float,re.findall(r'[0-9.]+',s.split('[')[1]))) for s in o[8:]]
    return (z,n,best,ivs)
with ThreadPoolExecutor(8) as ex:
  for s in sweeps:
    rr=[s[0]*24+k for k in range(24)] if len(s)==1 else [r for r in range(s[0],s[1]+1) if r in runs and r!=62]
    s=rr[0]
    z0=runs[rr[0]]; slope=b/math.sqrt(1-2*b*z0)/G
    if mode=='d':
        R=list(ex.map(delta_run,rr)); n=max(c for z,c,m,d in R); R=[t for t in R if t[1]>=n-1]
        feas=[]
        for i in range(4000):
            a=i/4000; lo=-1e9; hi=1e9; slack=float(__import__("os").environ.get("SLACK","2e-18"))
            for z,c,m,(dl,dh) in R:
                p_=K*G*(((a+slope*(z-z0))%1+0.5)%1-0.5)
                lo=max(lo,dl-p_-slack); hi=min(hi,dh-p_+slack)
            if lo<=hi: feas.append(a)
        ivs=[]
        for a in feas:
            if ivs and abs(a-ivs[-1][1]-1/4000)<1e-9: ivs[-1][1]=a
            else: ivs.append([a,a])
        if len(ivs)>1 and ivs[0][0]==0 and abs(ivs[-1][1]-3999/4000)<1e-9: ivs[0][0]=ivs[-1][0]-1; ivs.pop()
        if feas: print('sweep %d z0 %r n %d phase %s'%(s,z0,len(R),' '.join('[%.4f,%.4f]'%(a,b) for a,b in ivs)))
        else: print('sweep %d z0 %r n %d NO feasible phase'%(s,z0,len(R)))
        sys.stdout.flush()
    else:
        R=list(ex.map(ph_run,rr)); good=[]
        for i in range(2000):
            u0=i/2000; cnt=0
            for z,n,bst,ivs in R:
                if n-bst>1: continue
                f=(u0+slope*(z-z0))%1
                if any(a-0.004<=f<=bb+0.004 for a,bb in ivs): cnt+=1
            good.append((cnt,u0))
        mx=max(c for c,_ in good); us=[u for c,u in good if c==mx]; nv=sum(1 for z,n,bst,ivs in R if n-bst<=1)
        print('sweep %d z0 %r valid %d best %d u0 [%.4f..%.4f] npts %d'%(s,z0,nv,mx,min(us),max(us),len(us))); sys.stdout.flush()
