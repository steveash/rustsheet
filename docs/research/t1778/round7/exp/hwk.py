import mpmath as mp, pickle
exec(open('uerr.py').read().split("G=mp.mpf")[0])
mp.mp.prec=220
L2E=1/mp.log(2)
with mp.workprec(64): L2E64=+L2E
L=open('expdata.txt').read().split('\n')[:-1]; H=open('hw.txt').read().split('\n')[:-1]
R=pickle.load(open('kset_floor.pkl','rb')); inf={(r[0]):r for r in R if not all(r[4])}
cnt={}
for le,lh in zip(L,H):
    ah,eh,tag=le.split(); a=float.fromhex(ah)
    if a not in inf: continue
    p=lh.split(); f=ldhex(p[3]); u=ldhex(p[4])
    us=mp.power(2,f)-1; u0=r64(us); ul=ulp64(u0); k=int((u-u0)/ul)
    r=inf[a]; s=''.join('o' if x else '.' for x in r[4])
    okhw = r[4][4+k] if abs(k)<=4 else None
    cnt[okhw]=cnt.get(okhw,0)+1
    if not okhw or not r[4][4]: print(f'f={float(f):.5f} a={a:10.4f} {s} hw_k={k:+d} hw_ok={okhw} uerrRN={float((u0-us)/ul):+.3f}')
print(cnt)
