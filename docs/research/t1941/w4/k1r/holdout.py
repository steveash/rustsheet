import k1lib as K, numpy as np, sys, pickle
exec(open('latclimb.py').read().split("RUNS=K.runs")[0])
J=jac(K.N0,K.D0); V,f=basis(J)
sc,k=pickle.load(open(sys.argv[1],'rb')); NV=len(k); v=k@V[:NV]
print('move:',' '.join('%s%d:%+d'%(c,i,d) for (c,i),d in zip(idx,v) if d),' fchange %.4f'%np.max(np.abs(J@v)))
c0=K.gsimpy.Sim(K.N0,K.D0); c1=K.gsimpy.Sim(*apply(K.N0,K.D0,v))
for grp in sys.argv[2].split(';'):
    a=b=n=0
    for nm,y,g in K.runs(grp.split(',')):
        a+=c0.count(y,g); b+=c1.count(y,g); n+=len(y)
    print('%-22s n=%7d  AN2 %.4f -> %.4f'%(grp,n,a/max(n,1),b/max(n,1)))
