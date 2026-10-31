"""T-3269: the Stirling correction of ln Gamma at the shapes a = k/2, k = 1..39, as doubles.

correction(a) = ln Gamma(a) - ((a - 1/2) ln a - a + ln(2 pi) / 2), at 50 digits, rounded once to a
double and printed as the Rust table `STIRLING_CORRECTION` in
crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/excel_tail.rs (bit patterns,
so the table is exactly this output).

Run: python3 -I docs/research/t3269/stirling_correction.py  (needs mpmath)
"""

import struct

import mpmath as mp

mp.mp.dps = 50
for k in range(1, 40):
    a = mp.mpf(k) / 2
    c = mp.loggamma(a) - ((a - mp.mpf(1) / 2) * mp.log(a) - a + mp.log(2 * mp.pi) / 2)
    bits = struct.unpack("<Q", struct.pack("<d", float(c)))[0]
    print(f"    f64::from_bits(0x{bits >> 48:04X}_{(bits >> 32) & 0xFFFF:04X}_{(bits >> 16) & 0xFFFF:04X}_{bits & 0xFFFF:04X}), // a = {k / 2}: {float(c)!r}")
