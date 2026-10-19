# T-1941 W6: run-ladder phase unwrap (core by W4, ladctl4.py/ladjoint.py; for the q421-q433 readout).
# meas: list of (K, phi) pairs ordered rung by rung (pairs of +-s), K = (x0 - a)/2^-52, phi = per-rung demodulated
# phase (cycles, in (-1/2,1/2]) relative to the anchor run at the signed base-line frequency Fm. g0 = (dF/dx)/2 * 2^-52
# from the track slope (within +-10%). Returns f (line frequency offset at a, cycles/step) and g (so dF/dx = 2g/2^-52),
# and the residuals (reject the line if max |res| > 0.06 cycles).
import numpy as np
def unwrap(meas,g0):
    Ks=np.array([m[0] for m in meas],float); ph=np.array([m[1] for m in meas])
    unw=[]; f=0.0; g=g0
    for i in range(0,len(Ks),2):
        for q in range(i,min(i+2,len(Ks))):   # a trailing odd rung is unwrapped alone
            pred=f*Ks[q]+g*Ks[q]**2; unw.append(ph[q]+np.round(pred-ph[q]))
        kk=Ks[:i+2]; yy=np.array(unw); sc=np.max(np.abs(kk))
        A=np.vstack([kk/sc,(kk/sc)**2]).T; b=yy.copy()
        wprior=0.05/max(abs(g0)*sc*sc*0.02,1e-12)
        A=np.vstack([A,[0,wprior]]); b=np.concatenate([b,[wprior*g0*sc*sc]])
        c,*_=np.linalg.lstsq(A,b,rcond=None); f=c[0]/sc; g=c[1]/sc/sc
    res=np.array(unw)-(f*Ks+g*Ks**2)
    return f,g,res
def demod(e,F):
    k=np.arange(len(e)); w=np.hanning(len(e)); return np.sum(e*w*np.exp(-2j*np.pi*F*k))
