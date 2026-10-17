# T-1778: which one-parameter W families fit every sweep phase (reads phases_all.txt; needs mpmath).
import mpmath as m
m.mp.dps=60
G=m.mpf(2)**-53
ph=[(l.split()[0],m.mpf(l.split()[1]),float(l.split()[2])) for l in open('phases_all.txt')]
fams={
 'exp': lambda b,z: m.exp(-b*z),
 'recip': lambda b,z: 1/(1+b*z),
 'pade11': lambda b,z: (1-b*z/2)/(1+b*z/2),
 'lin': lambda b,z: 1-b*z,
 'sqrt': lambda b,z: m.sqrt(1-2*b*z),
 'exp2': lambda b,z: 2*m.exp(-b*z/2)-1,
 'cosh?': lambda b,z: 1/m.cosh(m.sqrt(2*b*z)) if z>0 else 1,
 'recipsq': lambda b,z: 1/m.sqrt(1+2*b*z),
}
tol=0.02
for name,f in fams.items():
    cands=[(m.mpf('0.4546'),m.mpf('0.4548'))]
    alive=True
    for nm,z,phi in ph:
        new=[]
        for a,b in cands:
            ua=(1-f(a,z))/G; ub=(1-f(b,z))/G
            if ua>ub: ua,ub=ub,ua
            for k in range(int(m.floor(ua-phi))-1,int(m.ceil(ub-phi))+2):
                t1,t2=k+phi-tol,k+phi+tol
                if t2<ua or t1>ub: continue
                # bisection for beta where u=t1 and u=t2 within [a,b]
                def solve(t):
                    lo,hi=a,b
                    for _ in range(200):
                        mid=(lo+hi)/2
                        if (1-f(mid,z))/G<t: lo=mid
                        else: hi=mid
                    return lo
                bl=solve(t1) if t1>ua else a; bh=solve(t2) if t2<ub else b
                if bh>bl: new.append((bl,bh))
        cands=new
        if len(cands)>200000: break
        if not cands: print(name,'dies at',nm); alive=False; break
    if alive: print(name,'SURVIVES',len(cands),[(m.nstr(a,20),m.nstr(b,20)) for a,b in cands[:3]])
