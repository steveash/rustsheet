"""Exact rounding helpers: values as mpmath mpf at high precision."""
import mpmath
mpmath.mp.prec=300
def round_bits(v,bits):
    """Round mpf v to `bits` significant bits, ties to even (no exponent range limits)."""
    if v==0: return mpmath.mpf(0)
    s,man,exp,bc=v._mpf_
    if bc<=bits: return v
    sh=bc-bits; q=man>>sh; r=man-(q<<sh); half=1<<(sh-1)
    if r>half or (r==half and q&1): q+=1
    return mpmath.mpf((s,q,exp+sh,q.bit_length()))
def rn53(v): return float(round_bits(v,53))
def rn64(v): return round_bits(v,64)
def ulp53(x):
    import math; return math.ulp(x)
