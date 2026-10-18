import numpy as np
b=np.concatenate([np.load('runs_R.npy'),np.load('stepmap_R.npy')]); x,lo,hi,cnt,run,erx,y,E=b.T
m=cnt==1; x,R=x[m],lo[m]
LD=np.longdouble
def f87(op,a,c):
    A=a.astype(LD); C=c.astype(LD)
    r={'*':A*C,'/':A/C}[op]; return r.astype(np.float64)
z87=f87('*',x,x)
ones=np.ones_like(x)
A_c={'x':x,'1/x (53)':1/x,'1/x (87)':f87('/',ones,x),'z87':z87,'1/z87':1/z87,'sqrt(z87)':np.sqrt(z87),'1/sqrtpi':np.full_like(x,1/np.sqrt(np.pi)),'2x':2*x,'x+1':x+1,'x*sqrt2':x*np.sqrt(2)}
def reach(target, A, form):
    # exists double r: form(A,r)==target ?
    if form=='A*r': r0=target/A; f=lambda r: f87('*',A,r)
    elif form=='r/A': r0=target*A; f=lambda r: f87('/',r,A)
    elif form=='A/r': r0=A/target; f=lambda r: f87('/',A,r)
    ok=np.zeros(len(target),bool)
    for k in range(-4,5):
        r=r0.copy()
        for _ in range(abs(k)): r=np.nextafter(r,np.inf if k>0 else -np.inf)
        ok|=f(r)==target
    return ok
Rc=np.nextafter(R,np.inf)
for name,A in A_c.items():
    for form in ['A*r','r/A','A/r']:
        ok=reach(R,A,form); okc=reach(Rc,A,form)
        print(f'{name:10s} {form:4s} R unreachable {(~ok).sum():6d}/{len(R)}   control {(~okc).sum():6d}')
