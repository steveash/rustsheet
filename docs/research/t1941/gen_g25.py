# T-1941 reviewer probes P1-P4 (HANDOFF section 9.4)
import json, math
CASES='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/gamwt/eval/suites/probes-numeric/cases'
def run_up(x0,n):
    xs=[x0]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],math.inf))
    return xs
def write(name,desc,cells):
    case={"format":"rustsheet-case/v1","description":desc,"tags":["M9","probe","numeric","gamma"],"expected":{},"sheets":[{"name":"Calc","cells":cells}]}
    json.dump(case,open(f'{CASES}/{name}.case.json','w'),indent=1)
# P1: K1 steps, 26 x 8192, D = GAMMA(A)
for i in range(26):
    y0=1.70+0.0103137*i+0.0031415; cells={}
    for r,x in enumerate(run_up(y0,8192),1): cells[f"A{r}"]=x; cells[f"D{r}"]=f"=GAMMA(A{r})"
    write(f"kmyst-gam-p1{i+1:02d}",f"Kernel-mystery probe (T-1941 reviewer P1): GAMMA G-kernel plateau steps across 1.70-1.96, 8,192 consecutive doubles from x0={y0!r} (y0 = 1.70 + 0.0103137*i + 0.0031415, i={i}). D = GAMMA(x).",cells)
# P2: (1.5,2] glue
for k,x0 in enumerate((1.52,1.66,1.81,1.97)):
    cells={}
    for r,x in enumerate(run_up(x0,16384),1):
        cells[f"A{r}"]=x; cells[f"B{r}"]=f"=GAMMALN.PRECISE(A{r})"; cells[f"C{r}"]=f"=LN(A{r})"; cells[f"E{r}"]=f"=GAMMALN.PRECISE(A{r}+1)"
        cells[f"F{r}"]=f"=GAMMALN.PRECISE(A{r}-1)"; cells[f"G{r}"]=f"=LN(A{r}-1)"; cells[f"H{r}"]=f"=GAMMA(A{r})"
    write(f"kmyst-gam-p2{k+1:02d}",f"Kernel-mystery probe (T-1941 reviewer P2): GAMMALN (1.5,2] glue, 16,384 consecutive doubles from x0={x0!r}. B = GAMMALN.PRECISE(x), C = LN(x), E = GAMMALN.PRECISE(x+1), F = GAMMALN.PRECISE(x-1), G = LN(x-1), H = GAMMA(x).",cells)
# P3: [3,8) glue
cells={}
for r,x in enumerate(run_up(3.37,16384),1):
    cells[f"A{r}"]=x; cells[f"B{r}"]=f"=GAMMALN.PRECISE(A{r})"; cells[f"E{r}"]=f"=GAMMALN.PRECISE(A{r}-1)"; cells[f"G{r}"]=f"=A{r}-1"; cells[f"F{r}"]=f"=LN(G{r})"
write("kmyst-gam-p301","Kernel-mystery probe (T-1941 reviewer P3): GAMMALN [3,8) glue, 16,384 consecutive doubles from x0=3.37. B = GAMMALN.PRECISE(x), E = GAMMALN.PRECISE(x-1), G = x-1, F = LN(G). Test B == fl(E + F).",cells)
cells={}
for r,x in enumerate(run_up(5.71,16384),1):
    cells[f"A{r}"]=x; cells[f"B{r}"]=f"=GAMMALN.PRECISE(A{r})"; cells[f"E{r}"]=f"=GAMMALN.PRECISE(A{r}-3)"; cells[f"G{r}"]=f"=A{r}-3"
    cells[f"H{r}"]=f"=G{r}*(G{r}+1)"; cells[f"I{r}"]=f"=H{r}*(G{r}+2)"; cells[f"F{r}"]=f"=LN(I{r})"
write("kmyst-gam-p302","Kernel-mystery probe (T-1941 reviewer P3): GAMMALN [3,8) glue, 16,384 consecutive doubles from x0=5.71. B = GAMMALN.PRECISE(x), E = GAMMALN.PRECISE(x-3), G = x-3, H = G*(G+1), I = H*(G+2) (ascending stored product), F = LN(I). Test B == fl(E + F).",cells)
# P4: legacy GAMMALN
cells={}
for r,x in enumerate(run_up(2.33,16384),1):
    cells[f"A{r}"]=x; cells[f"B{r}"]=f"=GAMMALN.PRECISE(A{r})"; cells[f"C{r}"]=f"=GAMMALN(A{r})"; cells[f"D{r}"]=f"=GAMMA.DIST(1,A{r},1,FALSE)"; cells[f"E{r}"]=f"=EXP(-B{r})"
write("kmyst-gam-p401","Kernel-mystery probe (T-1941 reviewer P4): legacy GAMMALN vs GAMMALN.PRECISE, 16,384 consecutive doubles from x0=2.33. B = GAMMALN.PRECISE(x), C = GAMMALN(x), D = GAMMA.DIST(1,x,1,FALSE), E = EXP(-B).",cells)
print('ok')
