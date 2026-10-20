import numpy as np
def remez(basis,f,x,ref):
    for it in range(40):
        A=np.array([[b(xi) for b in basis]+[(-1)**j] for j,xi in enumerate(ref)]); sol=np.linalg.solve(A,f(ref)); a=sol[:-1]; E=sol[-1]
        e=f(x)-sum(ai*b(x) for ai,b in zip(a,basis))
        idx=[0]+[i for i in range(1,len(x)-1) if (e[i]-e[i-1])*(e[i+1]-e[i])<=0 and e[i]!=e[i-1]]+[len(x)-1]
        blocks=[]
        for i in idx:
            if blocks and np.sign(e[i])==np.sign(e[blocks[-1]]):
                if abs(e[i])>abs(e[blocks[-1]]): blocks[-1]=i
            else: blocks.append(i)
        while len(blocks)>len(ref):
            blocks=blocks[1:] if abs(e[blocks[0]])<abs(e[blocks[-1]]) else blocks[:-1]
        new=x[blocks]
        if len(new)<len(ref): break
        if np.allclose(new,ref,atol=1e-9): break
        ref=new
    return a,abs(E),e
# usage: minimax rel approx of 1/x on [2,3] by degree-n poly q: a,E,e = remez([lambda x:x**k for k in 1..n+1], lambda x:x**0, grid, chebyshev_ref)
