"""Build probe1.case.json: BETA.DIST(x,a,b,FALSE) on irregular x grids and consecutive-double runs for 10 (a,b)."""
import json, math
CONF=[(0.5,7.0),(0.5,1.0),(1.0,7.0),(0.25,3.3),(2.5,3.5),(12.0,20.0),(0.5,20.0),(3.0,0.5),(1.5,1.0),(1.0,20.0)]
xs=[0.00137+0.000997713*k for k in range(1000)]
def run(x0,n):
    out=[x0]
    for _ in range(n-1): out.append(math.nextafter(out[-1],2))
    return out
xs+=run(0.0123456789,300)+run(0.7123456789,300)
sheets=[]
for j,(a,b) in enumerate(CONF):
    cells={}
    for k,x in enumerate(xs):
        cells[f"A{k+1}"]=x
        cells[f"B{k+1}"]=f"=BETA.DIST(A{k+1},{a!r},{b!r},FALSE)"
    sheets.append({"name":f"C{j}","cells":cells})
json.dump({"format":"rustsheet-case/v1","description":"T-2464 w1-density probe1: beta density on x grids for 10 shape pairs","tags":["M9","probe"],"sheets":sheets,"expected":{}},open("probe1.case.json","w"))
json.dump({"conf":CONF,"xs":xs},open("probe1.meta.json","w"))
print(len(xs)*len(CONF))
