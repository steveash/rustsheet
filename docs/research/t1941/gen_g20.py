import json, math
CASES='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/gamwt/eval/suites/probes-numeric/cases'
def run_up(x0,n):
    xs=[x0]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],math.inf))
    return xs
def run_below(x1,n):   # n consecutive doubles ending at x1
    xs=[x1]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],-math.inf))
    return xs[::-1]
runs=[]
for j in range(52,5,-1):
    c=1.3071+0.0377*(j%17)
    runs.append((f"2.5+c*2^-{j}",run_up(2.5+c*2.0**-j,2048)))
    runs.append((f"2.5-c*2^-{j}",run_below(2.5-c*2.0**-j,2048)))
for j in range(52,5,-1):
    c=1.7313+0.0291*(j%17)
    x1=3-c*2.0**-j
    if x1>=3: x1=math.nextafter(3.0,0)
    runs.append((f"3-c*2^-{j} (run ends there)",run_below(x1,2048)))
H=("Kernel-mystery probe (T-1941, W2 request): GAMMALN base-kernel epsilon ladders at a second anchor, 2,048 consecutive "
   "doubles per run, at x0 = 2.5 +/- c*2^-j (c = 1.3071+0.0377*(j mod 17)) and ending at 3 - c*2^-j (c = 1.7313+0.0291*(j mod 17)), j = 52..6.")
k=0
for i in range(0,len(runs),16):
    cells={}; row=1; desc=[]
    for lab,xs in runs[i:i+16]:
        desc.append(f"{lab}: x0={xs[0]!r} row={row}")
        for x in xs: cells[f"A{row}"]=x; cells[f"B{row}"]=f"=GAMMALN.PRECISE(A{row})"; row+=1
    k+=1
    case={"format":"rustsheet-case/v1","description":H+" Runs: "+"; ".join(desc)+". B = GAMMALN.PRECISE(x).",
          "tags":["M9","probe","numeric","gamma"],"expected":{},"sheets":[{"name":"Calc","cells":cells}]}
    json.dump(case,open(f'{CASES}/kmyst-gam-e6{k:02d}.case.json','w'),indent=1)
print(k,'cases',len(runs),'runs')
