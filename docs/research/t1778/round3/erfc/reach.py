import numpy as np, sys
a=np.concatenate([np.load('stepmap.npy'),np.load('runs.npy')])
x,y,E,z,rE,rT,eE,erx,run=a.T
m=y>2.3e-308*4  # normal
x,y,E=x[m],y[m],E[m]
r0=y/E
def reach(op):
    ok=np.zeros(len(y),bool)
    for k in range(-4,5):
        r=r0.copy()
        if k>0:
            for _ in range(k): r=np.nextafter(r,np.inf)
        elif k<0:
            for _ in range(-k): r=np.nextafter(r,-np.inf)
        ok|= (op(E,r)==y)
    return ok
f53=lambda E,r: E*r
f87=lambda E,r: (E.astype(np.longdouble)*r.astype(np.longdouble)).astype(np.float64)
for name,op in [('fl53(E*R)',f53),('fl53(fl64(E*R))',f87)]:
    ok=reach(op)
    print(name,'reachable',ok.sum(),'/',len(ok), 'unreachable',(~ok).sum())
    for lo,hi in [(0.5,1),(1,2),(2,4),(4,8),(8,16),(16,27)]:
        mm=(x>=lo)&(x<hi); print('   [%g,%g) unreach %d / %d'%(lo,hi,(~ok[mm]).sum(),mm.sum()))
# baseline: how many y's are unreachable generally? estimate via random alternative y +-1ulp
yy=np.nextafter(y,np.inf)
ok=np.zeros(len(y),bool)
for k in range(-4,5):
    r=yy/E
    for _ in range(abs(k)): r=np.nextafter(r,np.inf if k>0 else -np.inf)
    ok|=(E*r==yy)
print('control y+1ulp reachable', ok.sum(), 'unreach', (~ok).sum())
