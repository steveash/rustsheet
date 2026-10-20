# Exact-null lattices of integer-ulp coefficient moves. For a coefficient block (indices J of p or q) the move
# delta in Z^|J| changes the stage-K partial polynomial by g(x)=sum_j delta_j ulp(c_j) x^(K-j); we measure it in
# units of u=2^-54 (ulp of s4 / d3) on x in [2,3]. Reports: singular values of the sampled map, the ellipsoid
# estimate of the number of lattice points with RMS|g| <= eps*u, and the LLL-shortest move with its sup norm.
import sys, math, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import p as P0, q as Q0
def ulp(c): return 2.0**(math.floor(math.log2(abs(c)))-52)
u=2.0**-54; xs=np.linspace(2,3,401)
def lll(B, delta=0.99):   # B: list of basis vectors (rows, float); returns reduced basis and unimodular T (rows = coeffs)
    B=np.array(B,dtype=float); n=len(B); T=np.eye(n,dtype=object)
    for i in range(n):
        for j in range(n): T[i,j]=int(T[i,j])
    def gs(B):
        Q=np.zeros_like(B); mu=np.zeros((n,n))
        for i in range(n):
            Q[i]=B[i]
            for j in range(i):
                mu[i,j]=B[i]@Q[j]/(Q[j]@Q[j]); Q[i]-=mu[i,j]*Q[j]
        return Q,mu
    k=1
    while k<n:
        Q,mu=gs(B)
        for j in range(k-1,-1,-1):
            r=round(mu[k,j])
            if r: B[k]-=r*B[j]; T[k]-=r*T[j]; Q,mu=gs(B)
        if Q[k]@Q[k] >= (delta-mu[k,k-1]**2)*(Q[k-1]@Q[k-1]): k+=1
        else:
            B[[k,k-1]]=B[[k-1,k]]; T[[k,k-1]]=T[[k-1,k]]; k=max(k-1,1)
    return B,T
def analyse(name,c,J,K):
    A=np.array([[ulp(c[j])*x**(K-j)/u for j in J] for x in xs])   # samples x |J|, in u
    sv=np.linalg.svd(A/np.sqrt(len(xs)),compute_uv=False)        # RMS-normalised
    d=len(J); Vd=math.pi**(d/2)/math.gamma(d/2+1)
    print(f"{name}: block {J} measured at stage {K}; 1-ulp moves are {', '.join(f'{A[len(xs)//2,i]:.3g}' for i in range(d))} u at x=2.5")
    print(f"   RMS singular values (u per unit move): "+" ".join(f"{s:.3g}" for s in sv))
    for eps in (1e-3,1e-2,1e-1,1.0):
        print(f"   eps={eps:g} u: expected # of integer moves with RMS|g|<=eps: {Vd*eps**d/np.prod(sv):.3g}")
    B,T=lll([A[:,i]*1e3 for i in range(d)])
    norms=[np.max(np.abs(A@np.array(T[i],dtype=float))) for i in range(d)]
    o=np.argsort(norms)
    for i in o[:3]:
        v=np.array(T[i],dtype=float); print(f"   short move delta={[int(T[i][j]) for j in range(d)]} ulps: sup|g|={norms[i]:.3g} u, RMS={np.sqrt(np.mean((A@v)**2)):.3g} u")
analyse("N head p1..p3 at v4",P0,[1,2,3],4)
analyse("N head p1..p4 at s4",P0,[1,2,3,4],4)
analyse("D head q0..q2 at v3",Q0,[0,1,2],3)
analyse("D head q0..q3 at d3",Q0,[0,1,2,3],3)
analyse("N late p5..p9 at N",P0,[5,6,7,8,9],9)
analyse("D late q4..q8 at D",Q0,[4,5,6,7,8],8)
analyse("N p1..p9 at N",P0,list(range(1,10)),9)
print("late grid ratios ulp(p_k)/ulp(s_k) and output size of one s_k re-decision, x=2.5:")
x=2.5; s=P0[0]
for k in range(1,10): s=s*x+P0[k]
N=s; s=P0[0]
for k in range(1,10):
    s=s*x+P0[k]
    if k>=5: print(f"   k={k}: ulp(p)/ulp(s)={ulp(P0[k])/ulp(s):g}  1 ulp(s_k) -> {ulp(s)*x**(9-k)/ulp(N):.2f} ulp(N)")
