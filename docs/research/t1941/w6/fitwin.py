# T-1941 W6: exact-rational error of the known candidate sets vs the true function over a wide x range, to reveal
# the generator's fit interval (error small/equioscillating inside, growing outside) and criterion (rel/abs).
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import read_k2, read_k3
mp.mp.dps=50
N3,D3=read_k3(); P,Q=read_k2()
def ev(c_asc,x): return mp.polyval([mp.mpf(v) for v in c_asc[::-1]],x)
def show(name,num,den,f,xs):
    print('==',name)
    for x in xs:
        x=mp.mpf(x); v=ev(num,x)/ev(den,x); t=f(x)
        print('  x=%.3f  rel err %+.3e'%(float(x),float((v-t)/t) if t!=0 else float(v-t)))
show('K3 ss_fc A=N/D vs lnG/(x-1)',N3,D3,lambda z: mp.loggamma(z)/(z-1) if z!=1 else -mp.euler,
     [0.3,0.4,0.5,0.55,0.6,0.65,0.7,0.8,0.9,1.0,1.1,1.2,1.3,1.4,1.5,1.55,1.6,1.7,1.8,1.9,2.0,2.2,2.5])
show('K2 efC S=N/D vs lnG/(x-2)',P[::-1],Q[::-1],lambda z: mp.loggamma(z)/(z-2) if z!=2 else 1-mp.euler,
     [1.0,1.2,1.4,1.5,1.6,1.8,1.9,2.0,2.2,2.5,2.8,3.0,3.1,3.2,3.4,3.6,3.8,4.0,4.5,5.0])
