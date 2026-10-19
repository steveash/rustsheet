import json, math
CASES='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/gamwt/eval/suites/probes-numeric/cases'
H=("Kernel-mystery probe (T-1941, W4 request): GAMMA on [1,2), 63 runs of 4,096 consecutive doubles starting near "
   "y = 1 + j/64 with an irregular offset (line tracks of the G kernel's Horner stages).")
runs=[(1+j/64+0.000173*j+0.0000371,4096) for j in range(1,64)]
for c in range(16):
    cells={}; row=1; desc=[]
    for x0,n in runs[4*c:4*c+4]:
        x=x0; desc.append(f"x0={x0!r} n={n} row={row}")
        for i in range(n):
            cells[f"A{row}"]=x; cells[f"D{row}"]=f"=GAMMA(A{row})"; row+=1; x=math.nextafter(x,math.inf)
    case={"format":"rustsheet-case/v1","description":H+" Runs: "+"; ".join(desc)+". D = GAMMA(x).",
          "tags":["M9","probe","numeric","gamma"],"expected":{},"sheets":[{"name":"Calc","cells":cells}]}
    json.dump(case,open(f'{CASES}/kmyst-gam-y1{c+1:02d}.case.json','w'),indent=1)
print('ok')
