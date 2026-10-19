# T-1941: split unfilled long probe cases into parts of <= CHUNK rows at run boundaries (oracle fill time grows
# superlinearly with case size: 65,536-row cases take ~12 min vs ~50 s for 16,384). Part k of case X is X<letter>,
# rows renumbered from 1; the description records the original row offset.
import json, os, re, sys
D, RUN, CHUNK = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
for name in sys.argv[4:]:
    p = os.path.join(D, name + '.case.json')
    assert not os.path.exists(os.path.join(D, name + '.oracle.json')), name
    case = json.load(open(p)); cells = case['sheets'][0]['cells']
    rows = {}
    for k, v in cells.items():
        m = re.fullmatch(r'([A-Z]+)(\d+)', k); rows.setdefault(int(m[2]), {})[m[1]] = v
    n = max(rows); per = (CHUNK // RUN) * RUN
    for part, start in enumerate(range(1, n + 1, per)):
        q = os.path.join(D, f'{name}{"abcdefgh"[part]}.case.json'); assert not os.path.exists(q), q
        new = {}
        for r in range(start, min(start + per, n + 1)):
            for c, v in rows[r].items():
                new[f'{c}{r - start + 1}'] = v.replace(f'A{r})', f'A{r - start + 1})') if isinstance(v, str) else v
        c2 = dict(case); c2['sheets'] = [{'name': case['sheets'][0]['name'], 'cells': new}]
        c2['description'] = case['description'] + f' Part {"abcdefgh"[part]}: original rows {start}..{min(start + per, n + 1) - 1}.'
        json.dump(c2, open(q, 'w'), indent=1)
    os.remove(p)
print('ok')
