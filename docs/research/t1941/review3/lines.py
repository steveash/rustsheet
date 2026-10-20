# Base stored-Horner line frequencies (cycles per x-step of 2^-51) for grid_a at x, exact rationals.
from fractions import Fraction as F
import math, sys
p=[float.fromhex(h) for h in "0x1.5717994ef45d1p-51 0x1.5e5b73810edd1p-16 0x1.0ee2f118ad254p-9 0x1.572bf8683b589p-5 0x1.09fd1ad1da445p-2 0x1.f0a8c05397a9fp-2 -0x1.08305b47998b9p-6 -0x1.0ae34eb3cbad6p-1 -0x1.de852aee8c153p-3 -0x1.22260ed97843dp-6".split()]
q=[float.fromhex(h) for h in "0x1.03cadf2db0d9dp-18 0x1.52fc9914ab713p-11 0x1.5303658ef66c8p-6 0x1.bb43b464d3259p-3 0x1.c515177eb1755p-1 0x1.7b39fc9194438p+0 0x1.eddf2040d2bc2p-1 0x1.9b252b2fab347p-3 0x1.00000000000eap-7".split()]
def e2(v): return math.floor(math.log2(abs(float(v))))
def ulp(v): return F(2)**(e2(v)-52)
def al(f): f=f%1; return float(min(f,1-f))
step=F(2)**-51
def stages(c,x):
    S=F(c[0]); dS=F(0); out=[]
    for i in range(1,len(c)):
        P=S*x; dP=dS*x+S; S=P+F(c[i]); dS=dP; out.append((i,P,dP,S,dS))
    return out
x=F(sys.argv[1]) if len(sys.argv)>1 else F(2)
for nm,c in (("N",p),("D",q)):
    print(nm,"x=",float(x))
    for i,P,dP,S,dS in stages(c,x):
        fP=dP*step/ulp(P); fS=dS*step/ulp(S)
        print(f"  st{i}: P={float(P):+.5g} 2^{e2(P)} fP={al(fP):.7f} | S={float(S):+.5g} 2^{e2(S)} fS={al(fS):.7f} tri={al(fS/2):.7f}")
print("first-rounding lines of candidate early schemes at hand-off:")
print(" N: C1 (c4+c3x)' flat:",al(F(p[3])*step/ulp(p[4]+p[3]*x)), " C2 even E':",al((2*p[2]*x+4*p[0]*x**3)*step/ulp(p[4]+p[2]*x*x)), " C3 low-first (c4+c3x+c2x2)':",al((p[3]+2*p[2]*x)*step/ulp(p[4]+p[3]*x+p[2]*x*x)))
print(" D: C1:",al(F(q[2])*step/ulp(q[3]+q[2]*x)), " C2:",al((2*q[1]*x)*step/ulp(q[3]+q[1]*x*x)), " C3:",al((q[2]+2*q[1]*x)*step/ulp(q[3]+q[2]*x+q[1]*x*x)))
