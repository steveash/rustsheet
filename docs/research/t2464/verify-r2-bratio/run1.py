import sys, pickle
sys.path.insert(0, '.')
from vscore import *
import bratio_excel as bx
rows = vload.load_all() + vload.load_fresh()
keys = sorted({key(r) for r in rows})
print('distinct keys', len(keys), 'rows', len(rows))
cr = cr_all(keys)
vals, br = model_values(rows, bx.BEST)
pickle.dump((rows, vals, br), open('run1.pkl', 'wb'))
print('done')
