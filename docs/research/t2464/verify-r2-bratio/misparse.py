import sys, pickle, statistics as st
sys.path.insert(0, '.')
from vscore import *
rows, vals, br = pickle.load(open('run1.pkl', 'rb'))
cr = pickle.loads(CRF.read_bytes())
trows, tres = pickle.load(open('../r2-bratio/results.pkl', 'rb'))
tb = tres['BEST']
def rep(name, pairs, crs):
    n, e, u, w = tally(pairs); _, ce, cu, cw = tally(zip(crs, [p[1] for p in pairs]))
    sd = [ (m - x) / math.ulp(x) for m, x in pairs]
    print(f"| {name} | {n} | {e}/{u}/{w} | {ce}/{cu}/{cw} | med {st.median(sd):+.1f} sd {st.pstdev(sd):.1f} |")
print("| block | n | BEST ex/u4/w15 | CR ex/u4/w15 | model-excel ulps |\n|---|---|---|---|---|")
sel_t = [(i, r) for i, r in enumerate(trows) if r['set'] == 'w4p1' and r['a'] == 7.3]
rep('w4p1 (7.3, b) worker parse b=0.6', [(tb[i][0], r['excel']) for i, r in sel_t], [r['cr'] for _, r in sel_t])
sel = [(i, r) for i, r in enumerate(rows) if r['set'] == 'w4p1' and r['a'] == 7.3]
rep('w4p1 (7.3, b) truncated b=0.599999999999999', [(vals[i], r['excel']) for i, r in sel], [cr[key(r)] for _, r in sel])
sel_t = [(i, r) for i, r in enumerate(trows) if r['set'] == 'w2f' and 30.6 < r['a'] < 30.8 and r['b'] == 1.7]
rep('w2f a-sweep (30.7,1.7,0.8117) worker parse', [(tb[i][0], r['excel']) for i, r in sel_t], [r['cr'] for _, r in sel_t])
sel = [(i, r) for i, r in enumerate(rows) if r['set'] == 'w2f' and 30.6 < r['a'] < 30.8 and r['b'] == 1.7 and r['x'] == 0.8117]
rep('w2f a-sweep (30.7,1.7,0.8117) truncated', [(vals[i], r['excel']) for i, r in sel], [cr[key(r)] for _, r in sel])
# only the cells whose literal had >15 digits
mis = {(r['a']) for r in trows if r['set'] == 'w2f' and 30.6 < r['a'] < 30.8 and r['b'] == 1.7} 
sel_t = [(i, r) for i, r in enumerate(trows) if r['set'] == 'w2f' and 30.6 < r['a'] < 30.8 and r['b'] == 1.7 and r['x'] == 0.8117]
rep('w2f a-sweep x=0.8117 worker parse', [(tb[i][0], r['excel']) for i, r in sel_t], [r['cr'] for _, r in sel_t])
sel_t = [(i, r) for i, r in enumerate(trows) if r['set'] == 'w2f' and 30.6 < r['a'] < 30.8 and r['b'] == 1.7 and r['x'] == 0.8117 and len(repr(r['a']).replace('.', '').lstrip('0')) > 15]
rep('  worker, only the >15-digit literals', [(tb[i][0], r['excel']) for i, r in sel_t], [r['cr'] for _, r in sel_t])
sel_t2 = [(i, r) for i, r in enumerate(trows) if r['set'] == 'w2f' and 30.6 < r['a'] < 30.8 and r['b'] == 1.7 and r['x'] == 0.8117 and len(repr(r['a']).replace('.', '').lstrip('0')) <= 15]
rep('  worker, <=15-digit literals', [(tb[i][0], r['excel']) for i, r in sel_t2], [r['cr'] for _, r in sel_t2])
