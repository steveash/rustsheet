"""t0969-t-beta-link, t > 1 and y > 0.5 with z != fl(1-y): per bratio branch of K(n/2, 1/2, y, c), does T.DIST.RT
equal BETA.DIST(y)/2 (complement fl(1-y)), and which kernel input does the branch read (x0 = complement / y0 = y)?"""
import json, math, sys
from collections import Counter
sys.path.insert(0, '.')
from dispatch import bratio_branch, link_rows
cnt = Counter()
for t, n, g in link_rows():
    C, D, E, G, H = (g(k) for k in "CDEGH")
    if not (t > 1 and D > 0.5 and D <= 0.99) or G == 1 - D: continue
    br, ind, side = bratio_branch(n / 2, 0.5, D, 1 - D)
    cnt[(br, "E/2" if C == E / 2 else "not E/2")] += 1
for k in sorted(cnt): print(k, cnt[k])
print("--- bup+bgrat rows (t > 1, y <= 0.99): which composition of the BETA.DIST reference cells gives T.DIST.RT")
cnt = Counter(); n_ = 0
for t, n, g in link_rows():
    C, D, E, G, H, I = (g(k) for k in "CDEGHI")
    if not (t > 1 and D <= 0.99): continue
    br, ind, side = bratio_branch(n / 2, 0.5, D, 1 - D)
    if br != "bup+bgrat": continue
    n_ += 1
    forms = {"E/2": E / 2, "0.5-I/2": 0.5 - I / 2, "0.5-H/2": 0.5 - H / 2, "((0.5-I)+0.5)/2": ((0.5 - I) + 0.5) / 2,
             "(1-I)/2": (1 - I) / 2, "((0.5-H)+0.5)/2": ((0.5 - H) + 0.5) / 2}
    for k, v in forms.items(): cnt[k] += v == C
print(n_, "rows:", dict(cnt))
