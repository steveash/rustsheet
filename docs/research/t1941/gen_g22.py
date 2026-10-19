import json, math
CASES='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/gamwt/eval/suites/probes-numeric/cases'
def run_up(x0,n):
    xs=[x0]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],math.inf))
    return xs
def run_below(x1,n):
    xs=[x1]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],-math.inf))
    return xs[::-1]
runs=[]
for a in (2.125,2.25,2.375,2.625,2.75,2.875):
    for j in range(52,29,-1):
        c=1.3071+0.0377*(j%17)
        runs.append((f"{a}+c*2^-{j}",run_up(a+c*2.0**-j,2048)))
        runs.append((f"{a}-c*2^-{j} (run ends there)",run_below(a-c*2.0**-j,2048)))
H=("Kernel-mystery probe (T-1941, W2): GAMMALN base-kernel epsilon ladders at anchors 2.125, 2.25, 2.375, 2.625, 2.75, 2.875, 2,048 consecutive "
   "doubles per run at y0 = a + c*2^-j (run starts there) and a - c*2^-j (run ends there), c = 1.3071+0.0377*(j mod 17), j = 52..30.")
k=0
for i in range(0,len(runs),16):
    cells={}; row=1; desc=[]
    for lab,xs in runs[i:i+16]:
        desc.append(f"{lab}: x0={xs[0]!r} row={row}")
        for x in xs: cells[f"A{row}"]=x; cells[f"B{row}"]=f"=GAMMALN.PRECISE(A{row})"; row+=1
    k+=1
    case={"format":"rustsheet-case/v1","description":H+" Runs: "+"; ".join(desc)+". B = GAMMALN.PRECISE(x).",
          "tags":["M9","probe","numeric","gamma"],"expected":{},"sheets":[{"name":"Calc","cells":cells}]}
    json.dump(case,open(f'{CASES}/kmyst-gam-e8{k:02d}.case.json','w'),indent=1)
print(k,'cases',len(runs),'runs')
