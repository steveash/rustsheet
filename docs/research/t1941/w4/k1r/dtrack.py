import numpy as np, sys, gsk, collections
sys.path.insert(0,'../wt/docs/research/t1941/w6'); from synth import read_chain
LD=np.longdouble
N,D=read_chain('plain1.txt'); N=np.array(N); D=np.array(D)
runs=collections.defaultdict(list)
for l in open('k1R.txt'):
    q=l.split(); runs[q[5]].append((float.fromhex(q[0]),float.fromhex(q[1])))
for k,v in sorted(runs.items(),key=lambda kv:kv[1][0][0]):
    v=np.array(sorted(set(v))); x=v[:,0]; t=v[:,1]; cons=np.nextafter(x[:-1],np.inf)==x[1:]; br=np.where(~cons)[0]+1
    for s in np.split(np.arange(len(x)),br):
        if len(s)<8192: continue
        X=x[s][:8192]; T=t[s][:8192]; u=np.spacing(np.abs(T))
        if np.any(u!=u[0]): continue
        e=(T-gsk.G(N,D,0,0,X))/u; e-=e.mean(); han=np.hanning(8192); F=np.abs(np.fft.rfft(e*han))/han.sum()*2
        pred={}
        for ch,C in (('N',N),('D',D)):
            sv=np.full(8192,C[-1])
            for j in range(6,-1,-1):
                p=(sv.astype(LD)*X).astype(float); pred[ch+'P%d'%j]=((p[-1]-p[0])/8191/np.spacing(abs(p[4096])))%1
                sv=(p.astype(LD)+LD(C[j])).astype(float); pred[ch+'S%d'%j]=((sv[-1]-sv[0])/8191/np.spacing(abs(sv[4096])))%1
        j=np.argmax(F[5:-1])+5; f=j/8192; nm,fp=min(pred.items(),key=lambda q:min(abs(q[1]-f),abs(1-q[1]-f))); fp=min(fp,1-fp)
        print('%s y %.5f peak amp %.3f f %.5f nearest %s %.5f diff %+.5f'%(k,X[0],F[j],f,nm,fp,f-fp)); break
