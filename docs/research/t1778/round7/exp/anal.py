import mpmath as mp, sys
mp.mp.prec=300
L2E=1/mp.log(2)
with mp.workprec(64): L2E64=+L2E
def f64(v,rnd='n'):
    with mp.workprec(64): return mp.mpf(v) if False else mp.mpf(mp.libmp.normalize(*mp.mpf(v)._mpf_[:3],mp.mpf(v)._mpf_[3] if False else 0,64,rnd) if False else 0) 
def r64(v):
    with mp.workprec(64): return +v
def pos(v,E):
    # v (mpf) position in units of ulp53 relative to E
    e=mp.floor(mp.log(abs(E),2)); u=mp.mpf(2)**(e-52)
    return float((v-E)/u)
for l in open(sys.argv[1]):
    _,ah,eh,rh,tag=l.split()
    a=mp.mpf(float.fromhex(ah)); E=mp.mpf(float.fromhex(eh)); R=float.fromhex(rh)
    t=r64(a*L2E64); 
    print(f'{tag:9s} a={float(a):.6f} Excel-model={int((float(E)-R)/abs(R-float.fromhex(rh) or 1) if False else 0)} exact2^t64:{pos(mp.power(2,t),E):+.4f} true exp:{pos(mp.exp(a),E):+.4f} model={pos(mp.mpf(R),E):+.1f} terr(t64-a*L2E)/ulp64(t)={float((t-a*L2E)/ (mp.mpf(2)**(mp.floor(mp.log(abs(t),2))-63))):+.3f}')
