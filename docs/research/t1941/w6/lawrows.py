# T-1941 W6: assembler rows and predictions for a track under a law hypothesis (chain, law T/S/Pn<k>, j, sign, int).
import numpy as np
from stagelaws import stage_F
def law_key(law,j): return ('S%d'%j if law=='S' else 'T%d'%j) if law in ('T','S') else 'P%dn%s'%(j,law[2:])
def rows_for(ch,c,law,j,sg,kk,x,Fo,w=1e7,eoff=0):
    rows=[]
    for xv,fo in zip(x,Fo):
        sf=stage_F(np.asarray(c,float),np.array([xv]))
        if law in ('T','S'):
            terms=[(ch,j,law,1,int(np.floor(np.log2(abs(sf['S%d'%j][1][0]))))+eoff)]
        else:
            nn=int(law[2:]); terms=[(ch,j,'P',1,int(np.floor(np.log2(abs(sf['P%d'%j][1][0]))))+eoff),(ch,j+1,'S',-nn,int(np.floor(np.log2(abs(sf['S%d'%(j+1)][1][0]))))+eoff)]
        rows.append({'kind':'comb','terms':terms,'sign':sg,'int':kk,'x':[xv],'F':[fo],'w':w})
    return rows
def predict(N,D,ch,law,j,x):
    return stage_F(np.asarray(N if ch=='N' else D,float),np.asarray(x))[law_key(law,j)][0]
