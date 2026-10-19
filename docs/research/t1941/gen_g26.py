# T-1941 W-C probe P1b: 16,384 consecutive doubles on each side of AN2's K1 crossings
import json, math
CASES='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/gamwt/eval/suites/probes-numeric/cases'
def run_up(x0,n):
    xs=[x0]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],math.inf))
    return xs
k=131
for c in (1.0519,1.1799,1.2944,1.3403,1.3694,1.5445,1.7029,1.7295,1.7493,1.7807,1.7826,1.8888):
    for side,x0 in (("left",c-0.0004-16384*math.ulp(c)),("right",c+0.0004)):
        cells={}
        for r,x in enumerate(run_up(x0,16384),1): cells[f"A{r}"]=x; cells[f"D{r}"]=f"=GAMMA(A{r})"
        case={"format":"rustsheet-case/v1","description":f"Kernel-mystery probe (T-1941 W-C P1b): GAMMA G-kernel plateau height {side} of the AN2 crossing at {c}, 16,384 consecutive doubles from x0={x0!r}. D = GAMMA(x).",
              "tags":["M9","probe","numeric","gamma"],"expected":{},"sheets":[{"name":"Calc","cells":cells}]}
        json.dump(case,open(f'{CASES}/kmyst-gam-p{k}.case.json','w'),indent=1); k+=1
print(k-131,'cases')
