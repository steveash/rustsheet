"""Python wrapper for libinj.so and the injected-shift matched filter (ISMF) phase meter (T-1941 W5 part 5)."""
import ctypes, os, numpy as np
_lib = ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'libinj.so'))
Pd = ctypes.POINTER(ctypes.c_double)
_lib.res_inj.argtypes = [Pd, ctypes.c_long, Pd, ctypes.c_int, Pd, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int, Pd, ctypes.c_int, Pd]
def _c(a): return np.ascontiguousarray(a, dtype=np.float64)
def res_inj(x, N, D, ch, j, kind, phis):
    x = _c(x); N = _c(N); D = _c(D); phis = _c(phis); out = np.empty((len(phis), len(x)))
    _lib.res_inj(x.ctypes.data_as(Pd), len(x), N.ctypes.data_as(Pd), len(N), D.ctypes.data_as(Pd), len(D), ch, j, kind, phis.ctypes.data_as(Pd), len(phis), out.ctypes.data_as(Pd))
    return out
PH = np.arange(32)/32.0
def phase(x, b, N, D, ch, j, kind, base=None):
    """ISMF: returns (best phi, peak score, contrast). score(phi) = <d, s_phi> / ||s_phi||, d = b - base, s_phi = sim_phi - base.
    Phases are relative to the candidate (phi = 0 is the candidate itself)."""
    R = res_inj(x, N, D, ch, j, kind, PH)
    base = R[0] if base is None else base
    u = np.spacing(np.abs(base)); d = (b-base)/u; S = (R-base)/u
    nrm = np.sqrt((S*S).sum(1)); sc = (S@d)/np.maximum(nrm, 1e-9); sc[0] = 0.0
    i = int(np.argmax(sc)); return PH[i], sc[i], sc[i]/max(np.sqrt((d*d).sum()), 1e-9)
