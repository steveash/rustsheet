# T-1941 review (Fable 5.1): (1.5,2] composition variants with grid_a as the K2 base. Score exact matches of B on (1.5,2].
import sys; sys.path.insert(0,'.')
from k23lib import *
x,B,C,cs=load_rows_B(1.5000000001,2.0000000001)
ok=~np.isnan(C); x=x[ok]; B=B[ok]; C=C[ok]
xp53=st(L(x)+L(1.0)); xp64=L(x)+L(1.0)
def chain64(c_desc,xx,store=True):
    s=np.full(xx.shape,L(c_desc[0]),dtype=L)
    for ci in c_desc[1:]:
        p=s*xx; 
        if store: p=L(st(p))
        s=p+L(ci)
        if store: s=L(st(s))
    return s
res={}
for argname,xa in (('fl(x+1)',L(xp53)),('x+1 exact64',xp64)):
    for store in (True,False):
        S64=chain64(P2,xa,store)/chain64(Q2,xa,store)   # 64-bit quotient
        S53=L(st(S64))
        m64=xa-L(2.0); m53=L(st(m64))
        for pn,prod in (('st((x-1)*S53)',L(st(m53*S53))),('rn64((x-1)*S53)',m53*S53),('rn64(m*S64)',m64*S64),('st(m*S64)',L(st(m64*S64)))):
            for gl in ('stored','64'):
                out=st(prod-L(C))
                key=(argname,'stages stored' if store else 'stages 64',pn)
                res[key]=(out==B).mean()
for k,v in sorted(res.items(),key=lambda kv:-kv[1])[:12]: print('%.4f'%v,k)
