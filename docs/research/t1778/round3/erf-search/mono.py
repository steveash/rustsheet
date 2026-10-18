import sys, bisect
rows=[]
for l in open(sys.argv[1]):
    a=l.split(); z=float.fromhex(a[0]); sb=int(a[3]); rows.append((z,sb+int(a[1]),sb+int(a[2]),int(a[4]),int(a[5]),sb))
rows.sort()
# max monotone-feasible subset via DP with segment tree over compressed values
vals=sorted(set([r[1] for r in rows]+[r[2] for r in rows]))
idx={v:i for i,v in enumerate(vals)}; M=len(vals)
# g as array with lazy ops is heavy; use simple O(n log n) via "patience" style? Fallback: greedy count of violations
viol=0; mx=-1; bad=[]
for r in rows:
    if r[1]>mx: mx=r[1]
    if mx>r[2]: viol+=1; bad.append(r)
print('points',len(rows),'prefix-max violations',viol)
# reverse view
mn=1<<62; v2=0
for r in reversed(rows):
    if r[2]<mn: mn=r[2]
    if mn<r[1]: v2+=1
print('suffix-min violations',v2)
for r in bad[:10]: print(r)
