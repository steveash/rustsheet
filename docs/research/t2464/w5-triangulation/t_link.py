"""t0969-t-beta-link: T.DIST.RT (C) vs BETA.DIST at the worksheet's y (E), z (H) and 1-y (I)."""
import json, math, sys
from collections import Counter
sys.path.insert(0, '.')
from dispatch import bratio_branch, link_rows
cnt = Counter()
for t, n, g in link_rows():
    C, D, E, G, H, I = (g(k) for k in "CDEGHI")
    y, z = D, G
    zy = 1 - y
    a, b = n / 2, 0.5
    br, ind, side = bratio_branch(a, b, y, z)
    e_ok = C == E / 2
    h_ok = C == 0.5 - H / 2
    i_ok = C == 0.5 - I / 2
    key = ("t>1" if t > 1 else "t<=1", "y>0.5" if y > 0.5 else "y<=0.5", "z==1-y" if z == zy else "z!=1-y")
    cnt[key + (("E" if e_ok else "") + ("H" if h_ok else "") + ("I" if i_ok else "") or "none",)] += 1
for k in sorted(cnt): print(k, cnt[k])
print("--- t>1 cells where C != E/2")
for t, n, g in link_rows():
    C, D, E, G, H, I = (g(k) for k in "CDEGHI")
    if t > 1 and C != E / 2:
        br, ind, side = bratio_branch(n / 2, 0.5, D, G)
        print(f"t={t} n={n} y={D:.6f} z==1-y:{G == 1 - D} {br} ind={ind} side={side} C-E/2={(C - E/2)/math.ulp(C):+.0f}ulp C-(0.5-H/2)={(C-(0.5-H/2))/math.ulp(C):+.0f}")
