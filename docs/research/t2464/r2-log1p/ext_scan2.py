"""ext_scan over every subset of {t2, p, term, s, rs} of size k (argv[1]) on -r > 0.1 rows.  Usage: ext_scan2.py k"""
import itertools
import sys

import ext_scan as E
import gen_scan as G

rows = [x for x in G.load_rows() if -x[1] > 0.1]
for ext in itertools.combinations(["t2", "p", "term", "s", "rs"], int(sys.argv[1])):
    per, half, miss = G.score(E.make(set(ext)), rows)
    print("%-24s even %d/%d odd %d/%d" % (",".join(ext) or "none", *half[0], *half[1]), flush=True)
