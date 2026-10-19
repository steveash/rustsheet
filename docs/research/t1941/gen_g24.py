import json, math
CASES='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/gamwt/eval/suites/probes-numeric/cases'
def run_up(x0,n):
    xs=[x0]
    for _ in range(n-1): xs.append(math.nextafter(xs[-1],math.inf))
    return xs
runs=[]
for a in (1.125,1.375,1.625,1.875):
    for j in range(46,21,-1):
        c=1.2913+0.0419*(j%13)
        runs.append((f"{a}+c*2^-{j}",run_up(a+c*2.0**-j,8192)))
H=("Kernel-mystery probe (T-1941, W4): GAMMA G-kernel long epsilon ladders at anchors y = 1.125, 1.375, 1.625, 1.875, 8,192 consecutive "
   "doubles per run starting at y0 = a + c*2^-j, c = 1.2913+0.0419*(j mod 13), j = 46..22 (weak-stage lines).")
k=0
for i in range(0,len(runs),4):
    cells={}; row=1; desc=[]
    for lab,xs in runs[i:i+4]:
        desc.append(f"{lab}: x0={xs[0]!r} row={row}")
        for x in xs: cells[f"A{row}"]=x; cells[f"D{row}"]=f"=GAMMA(A{row})"; row+=1
    k+=1
    case={"format":"rustsheet-case/v1","description":H+" Runs: "+"; ".join(desc)+". D = GAMMA(x).",
          "tags":["M9","probe","numeric","gamma"],"expected":{},"sheets":[{"name":"Calc","cells":cells}]}
    json.dump(case,open(f'{CASES}/kmyst-gam-e10{k:02d}.case.json','w'),indent=1)
print(k,'cases',len(runs),'runs')
