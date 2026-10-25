"""Build data.tsv: one row per (x,a,b) with Excel's cdf and (when present) pdf at the same point, plus the exact cdf
and pdf (mpmath 40 digits), and the branch (direct: x <= a/(a+b)).  Sources: ../beta_exact.tsv (A=0,B=1 cells) and
w2-forensics' xruns/absweep (exact_w2f.tsv)."""
import csv, collections, mpmath
mpmath.mp.dps = 40
pts = collections.OrderedDict()
def add(src, blk, x, a, b, cum, excel, exact):
    k = (x, a, b)
    d = pts.setdefault(k, dict(src=src, blk=blk))
    d['cdf' if cum else 'pdf'] = excel
    d['cdfx' if cum else 'pdfx'] = exact
for r in csv.DictReader(open('../beta_exact.tsv'), delimiter='\t'):
    add('old', r['case'].split('/')[-1].replace('.case.json', ''), float(r['x']), float(r['a']), float(r['b']), r['cum'] == '1', r['excel'], r['exact'])
for r in csv.DictReader(open('../w2-forensics/exact_w2f.tsv'), delimiter='\t'):
    if 'unit-shape' in r['case']: continue
    i = int(r['cell'].split('!A')[1]) - 1
    blk = r['case'].replace('t2464-w2f-', '') + ':' + str(i // 2048 if 'xruns' in r['case'] else i // 1024)
    add('w2', blk, float(r['x']), float(r['a']), float(r['b']), r['cum'] == '1', r['excel'], r['exact'])
with open('data.tsv', 'w') as f:
    f.write('src\tblk\tx\ta\tb\tbranch\tcdf\tpdf\tcdfx\tpdfx\n')
    for (x, a, b), d in pts.items():
        br = 'direct' if x <= a / (a + b) else 'reflected'
        if 'pdfx' not in d:
            X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
            d['pdfx'] = mpmath.nstr(X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B), 30)
        if 'cdfx' not in d:
            d['cdfx'] = mpmath.nstr(mpmath.betainc(mpmath.mpf(a), mpmath.mpf(b), 0, mpmath.mpf(x), regularized=True), 30)
        f.write(f"{d['src']}\t{d['blk']}\t{x!r}\t{a!r}\t{b!r}\t{br}\t{d.get('cdf','nan')}\t{d.get('pdf','nan')}\t{d['cdfx']}\t{d['pdfx']}\n")
print(len(pts))
