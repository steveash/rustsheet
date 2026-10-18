import sys,struct
S='/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad'
files=['tiny','zs3','ws','wla','wk','wl2','wl','runs','xc','xc2','wb']
paths=[f'{S}/c/{f}.txt' for f in files]+[f'{S}/inv2/probes/rl.txt']
out=open('pts.bin','wb'); n=0
for fi,p in enumerate(paths):
    for line in open(p):
        a=line.split()
        if len(a)<3: continue
        x=float.fromhex(a[1]) if a[1].startswith(('0x','-0x')) else float(a[1])
        y=float.fromhex(a[2]) if a[2].startswith(('0x','-0x')) else float(a[2])
        if abs(x)>=0.5: continue
        out.write(struct.pack('<ddii',x,y,fi,int(a[0]))); n+=1
print(n)
