# T-1778 round 7: ERFC tail [4, 26.55], pinned Horner stages

ERFC(x) = fl53(fl64(E*R)), with E = Excel's EXP(-fl53(fl64(x*x))). The branch is at x = 4. R = fl(N/D) with
Horner in x, and every operation is stored as fl53(fl64(.)).

## Numerator (held-out verified)

| Stage | Constant |
|---|---|
| P = fl(a*x) | a = 0x1.4dc756d76135p-2 |
| Q = fl(P + b) | b = 0x1.6ca0f8ecfc798p+1 |
| T = fl(Q*x); V = fl(T + b2) | b2 = 0x1.0191f83634b97p+4 |
| W = fl(V*x); X3 = fl(W + b3) | b3 = 0x1.d936f5138efd9p+5 |
| Y = fl(X3*x); X4 = fl(Y + b4) | b4 = 0x1.396f0fc8e5c9cp+7 |
| Z = fl(X4*x); X5 = fl(Z + b5) | b5 = 0x1.26c409d384172p+8 (equivalent on data within [...84171, ...84173]) |
| Z5 = fl(X5*x); X6 = fl(Z5 + b6) | b6 = 0x1.80fb2b5716d18p+8 (equivalent on data within [...16d11, ...16d1f]) |
| Z6 = fl(X6*x); X7 = fl(Z6 + b7) | b7 = 0x1.3da266cd039e0p+8 (equivalent on data within [...03981, ...039ff]) |
| Z7 = fl(X7*x); N = fl(Z7 + b8) | b8 OPEN |

There is no ninth product, so deg N = 8.

## Denominator

| Stage | Constant |
|---|---|
| fl(beta*x) | beta = 0x1.27cdbb941913dp-2 |
| s1 = fl(. + b_D) | b_D = 0x1.4324d3ef0f238p+1, exact (three independent methods; crossings at x = 0x1.2f462bee491c9p+4 for 8 and 0x1.46e30db474374p+2 for 4) |
| fl(s1*x) | present; the step after it is not a stored add followed by a multiply (open) |

alpha/beta ≈ 2/sqrt(pi), so D's degree is N's plus 1.

## Scripts

- `zrun.py`: the numerator chain functions with all constants (paths inside point at the scratch data).
- `lib.py`: exact R recovery.
- `gtest.py`: the global stage regressor.
- `xstar.py`: exact crossing positions.
