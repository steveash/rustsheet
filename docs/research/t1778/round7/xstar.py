from lib import *
from gen import toi, fromi
def first_x(coef,T,x0):
    """smallest double x with fl6453(coef*x) >= T, search near x0"""
    x=x0
    f=lambda x: fl6453(F(coef)*F(x))>=T
    # walk
    step=1<<20
    i=toi(x0)
    while f(fromi(i)): i-=step
    while not f(fromi(i)): i+=step
    lo,hi=i-step,i  # f(lo) false, f(hi) true
    while hi-lo>1:
        m=(lo+hi)//2
        if f(fromi(m)): hi=m
        else: lo=m
    return fromi(lo),fromi(hi)
bD=float.fromhex('0x1.4324d3ef0f238p+1')
print('b_D',bD.hex(),repr(bD),'grid ok',(F(bD)*2**48).denominator==1)
lo,hi=first_x(BETA,F(8)-F(bD),18.9546317394)
print('x*_D8',hi.hex(),repr(hi),' prev',lo.hex(),repr(lo))
print('  fl(beta*prev)=',float(fl6453(F(BETA)*F(lo))).hex(),' fl(beta*x*)=',float(fl6453(F(BETA)*F(hi))).hex(),' 8-b=',(8-bD).hex())
# crossing at 4: w=fl(yD+b)>=4 iff yD+b >= 4-2^-52  -> threshold T4
T4=F(4)-F(bD)-F(1,2**52)
lo4,hi4=first_x(BETA,T4,5.107608248)
print('x*_D4',hi4.hex(),repr(hi4),' prev',lo4.hex(),repr(lo4))
for xx in (lo4,hi4):
    yv=fl6453(F(BETA)*F(xx)); print('  y',float(yv).hex(),'fl(y+b)=',float(fl(yv+F(bD))).hex())
bN=float.fromhex('0x1.6ca0f8ecfc798p+1')
loN,hiN=first_x(ALPHA,F(8)-F(bN),15.8037948)
print('x*_N8 (claimed b_N)',hiN.hex(),repr(hiN),'prev',repr(loN))
for xx in (loN,hiN):
    yv=fl6453(F(ALPHA)*F(xx)); print('  y',float(yv).hex(),'fl(y+b)=',float(fl(yv+F(bN))).hex())
