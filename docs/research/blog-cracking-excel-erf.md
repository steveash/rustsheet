# We spent two days reverse-engineering one Excel function. Here's what we found.

*How RustSheet's agents matched Excel's ERF and ERFC to the last bit, using only the numbers Excel gives back.*

---

## The problem: "close enough" isn't the job

RustSheet is a spreadsheet engine written in Rust, and its target is blunt: **same answers as Microsoft Excel,
down to the last bit**. Not "agrees to 10 decimal places". The same 64-bit double, every time.

Most of the time that's tedious but doable. Then you meet a function like `ERFC`.

`ERF` and `ERFC` are the **error function** and its complement. If you've ever computed the chance that a
normally distributed value lands within some range, you've used them, perhaps without knowing. `NORM.S.DIST`,
`CHISQ.DIST.RT` and a handful of other statistics functions are built on top of them. They're the bell curve's
plumbing.

There's no closed formula for erf. Every library approximates it, and every library does it differently. Ours
agreed with Excel to about 15 significant digits, which is excellent by any normal standard. It still differed in
the last binary digit on a large share of inputs. For a spreadsheet that's a visible bug: totals drift, `=A1=B1`
says FALSE, and the person comparing the two programs files a ticket.

So we set out to find out *exactly* what Excel does.

## Why this is hard

We have no source code. We didn't disassemble anything (that's off the table for us). What we have is an
**oracle**: we can send Excel formulas and read back the results. So the whole game is to treat Excel as a black
box and infer the algorithm from its outputs.

The first catch is that "the algorithm" includes **every rounding step**. A double has 53 bits of precision. Each
time a program multiplies or adds, the exact result gets rounded to fit. A formula like `a*x + b` computed in two
steps rounds twice, and the *order* of operations changes which way the last bit falls. Two programs can use the
same math, the same formula and the same constants, and still disagree in the last bit because one wrote
`(a*x)+b` and the other `b+(x*a)`.

The second catch is weirder. Excel's numeric core is old, and it does arithmetic on the **x87 floating-point
unit**, the math coprocessor Intel introduced in 1980. The x87 computes internally with **64-bit mantissas** (80-bit
registers), not 53. When a value is saved to memory it gets rounded *again*, down to a normal double. So every
operation in Excel is rounded twice: first to 64 bits, then to 53. We wrote this as `fl53(fl64(x))`, and it
turned out to matter for almost every bit we chased.

So to match Excel bit for bit we needed:
1. the same mathematical approximation (which formula, which degree, which coefficients),
2. the same coefficients **to the exact bit**,
3. the same order of operations,
4. the same double rounding at every step.

Getting any one of them wrong turns the output into noise in the last bit.

## Act 1: the textbook tour (none of it worked)

We started where any engineer would: maybe Excel uses a famous published algorithm. We tried them all:
- W. J. Cody's 1969 rational approximations (the classic, used almost everywhere);
- Cephes, Boost, fdlibm (what most C libraries use), SLATEC;
- Hart's 1968 *Computer Approximations* tables;
- Numerical Recipes' Chebyshev version;
- LibreOffice, HyperFormula, formula.js and a dozen other spreadsheet clones.

Each one matched somewhere between 16% and 40% of Excel's outputs exactly. That's the rate you get when the math
is right but the bits aren't. Not one of them was Excel's code. Clue number one: **Excel has its own private
implementation.**

## Act 2: let the rounding errors talk

If you can't read the code, read the noise.

Here's the key trick. Take 65,536 *consecutive* doubles, `x, x+1ulp, x+2ulp, ...` (an ulp, "unit in the last
place", is the smallest step a double can take), and ask Excel for ERFC at each one. Compare each answer with the
mathematically exact value. The tiny errors aren't random. Every time an intermediate value inside Excel's
algorithm gets rounded, it leaves a **sawtooth pattern** in the error as you sweep x. Different internal steps
make sawtooths at different frequencies.

So we ran Fourier transforms on rounding errors. The spectrum is a fingerprint of the algorithm's internal
structure: how many steps it takes, roughly what values pass through each, and where they cross powers of two.
It's like working out what's inside a sealed box by tapping it and listening to the echo.

We sent Excel millions of these probes. By the end there were over 10 million oracle points.

## Act 3: small x falls

The small-x region (0 ≤ x ≤ 0.5) cracked first, and it set the pattern for everything after.

Excel computes erf there as a **rational function**, one polynomial divided by another, in z = x²:

    erf(x) ≈ x · P(z) / Q(z)

The fingerprint said P was a 4th-degree polynomial and Q 5th-degree. Both are evaluated with **Horner's method**
(`((c4*z + c3)*z + c2)*z + ...`), and every intermediate step is rounded to a double the x87 way.

The structure didn't pin down the coefficients, though. To get those, we turned each output into a constraint. If
Excel returned *this* exact double, the hidden coefficients must lie within a certain tiny interval. Stack up
hundreds of thousands of such constraints and you get a **linear programming** problem: find coefficients that
satisfy all of them at once.

Result: **775,272 out of 775,272** points matched exactly, plus 35,718 points that were never used in the fit. 🎉

One odd detail: none of the textbook constructions reproduce those coefficients either. They're *close* to a
standard best-fit polynomial, but not to the bit.

## Act 4: the tail, and the EXP detour

For large x (4 < x ≤ 26.55) we found that Excel computes

    ERFC(x) = EXP(-x²) × N(x) / D(x)

But the `EXP` in that formula is **Excel's own EXP**, the worksheet function's implementation, so its exact bits
matter. Which raised an awkward question: is Excel's EXP itself exact?

**No!** We had to reverse-engineer it too. Excel's `EXP(a)` turns out to compute `2^(a·log₂e)` with x87 tricks:
split off the integer part, run the fractional part through the x87's built-in `F2XM1` instruction (it computes
2^f − 1), add 1 back, and take a reciprocal for negative fractions. It runs about 2% low by one ulp at large
inputs. Our model matches **2,139,591 of 2,139,684** cells. The last 93 are cases where the true value sits almost
exactly halfway between two representable numbers, and Excel's `F2XM1` rounds them differently from the Intel
chip we tested on. Our best guess is that the Excel machines answering our queries run on a different CPU (AMD?),
so those 93 cells may depend on the hardware. Yes, really.

With EXP modelled, we pinned the tail coefficients one stage at a time. We bisected the exact x where each
intermediate crosses a power of two, measured the phase of each sawtooth, and peeled stages from the top down.
That gave **1,540,475 of 1,540,477** points. The last two? We'll come back to those.

## Act 5: the middle, our white whale

Only **0.5 < x ≤ 4** was left, and it nearly beat us.

**Wrong turn #1: a (7,7) rational.** A least-squares fit said N and D were both degree 7, with a mysteriously tiny
top coefficient and a strange scale factor of 1.020495 hiding in the spectrum. We spent many hours on this.
Pinning it down kept failing in confusing ways: the top stages seemed "invisible", too faint to measure.

**Wrong turn #2: Cody's layout.** One worker found two strong spectral lines that matched Cody's classic (8,8)
layout at a scale of 1.8314. That was exciting for about an hour.

**The plot twist:** a worker tried to work out *how Excel's team would have generated* the coefficients. It
reproduced the tail's coefficients almost exactly with a standard **Remez minimax fit**, the textbook method for
"the best polynomial approximation", scaled by 128. Applied to the middle, the same recipe said the answer was a
**(9,10)** rational, not (7,7). Our spectral test, calibrated on the solved tail, scored the (9,10) at 0.68 and
every other shape at chance. The (7,7), with its tiny coefficient and its magic 1.020495, had all been an artefact
of forcing the wrong shape into a least-squares fit. The Cody hits were a coincidence.

**Wrong turn #3: false peaks.** Workers then started pinning the coefficients one at a time and found "exact"
matches with huge statistical confidence (z = 86!). These matches **contradicted each other**. It turned out all
our probe runs near x = 4 started at multiples of 1/128. On that regular grid, entire families of *wrong*
coefficients produce the same rounding pattern. The low bits were right and the high part was wrong. Lesson
learned: start your probes at messy, irregular positions.

**Wrong turn #4: searching in the wrong place.** Every search assumed Excel's coefficients sat within about 10,000
ulps of the best-fit solution. They were up to **130 million ulps** away. They aren't the textbook coefficients.
They're a cousin that computes almost exactly the same function (to about 1 part in 10²⁰) through very different
numbers. There's a "flat valley" of coefficient sets that all compute nearly the same curve, and Excel's sits
somewhere else in that valley.

## Act 6: the crack

The breakthrough came from a worker that stopped searching and started **solving**.

Each sawtooth's phase depends *linearly* on how far each hidden coefficient is from a guess, measured in whole
ulps. So instead of guess-and-check, it wrote down about 850 equations ("this phase implies this combination of
integer offsets") and solved them all at once with a **lattice reduction** algorithm (LLL, via the fpylll library).
That's the same family of math used to break weak cryptography, here aimed at a spreadsheet.

One solve took the match rate from **31% to 85.5%**. Then:
- a chain solve walked up the formula to fix the top coefficients;
- a second worker, pinning coefficients one at a time on the new irregular-start probes, independently landed on
  the same top denominator values to within a few ulps;
- a final brute-force search polished the last few bits.

**5,694,938 out of 5,694,943.** The 876,544 newest points, which the solve had never seen, matched 100%.

## The last five points (and the tail's last two)

Every miss, all seven of them across the middle and the tail, matched *perfectly* when we moved the input by one
ulp. The culprit wasn't the math at all: it was **Excel's file parser**. When Excel reads a number like
`2.4500000000004998` from an `.xlsx` file, it very occasionally rounds the decimal to the wrong neighbouring
double, about once per million inputs. Our model was right. Excel computed ERFC of a slightly different number
than the one we wrote.

So ERF and ERFC are now **bit-exact on every input we've probed**.

## Why would Excel do it this way? (Speculation corner)

None of this is documented, so here's our informed guessing:

- **It's old code, and it was good code.** Excel shipped in 1985, when the x87 coprocessor was *the* way to do
  fast floating point. Doing math in 80-bit registers was a feature then: extra precision for free. The double
  rounding is a side effect of that design, frozen in place.
- **Compatibility is forever.** Once millions of spreadsheets depend on exact results, changing the last bit is a
  breaking change. Someone's audited financial model would come out different. So the 1980s or 1990s
  implementation is probably still in there, wrapped carefully, x87 quirks and all.
- **They ran their own fits.** The break points (0.5 and 4) are Cody's, so someone clearly read the 1969 paper. But
  the coefficients come from a custom Remez fit, likely in some extended-precision tool, with its own rounding and
  convergence noise. That would explain why the numbers sit in a different spot of the "flat valley" than a modern
  90-digit fit.
- **Their own EXP, too.** Excel apparently doesn't call the C runtime's `exp`. Back then C libraries varied wildly
  between compilers and platforms (Mac and Windows!). Writing your own, straight onto x87 instructions, was the
  only way to get identical answers everywhere.

## What we took away

1. **Read the noise.** Rounding errors are a fingerprint, and Fourier transforms make them readable.
2. **Model the dependencies first.** We couldn't crack ERFC until we'd cracked EXP.
3. **Least squares can lie.** Force the wrong shape and you get beautiful, convincing artefacts.
4. **Probe at irregular points.** Regular grids create false answers.
5. **Solve, don't search.** Turning the problem into linear algebra (LP, then lattices) beat brute force every time.
6. **Test your methods on fake data first.** Every technique that worked had first recovered a known answer from
   synthetic data.
7. **Parallel, diverse attacks win.** The crack came from workers using *different* methods cross-checking each
   other, not from one method run harder.

We wrote all of this up as a playbook (`docs/research/excel-kernel-playbook.md`) because there are more
kernels to crack: the inverse normal, the beta distribution, the gamma function. They were very likely written by
the same team, in the same style, around the same time. Next time we know which rabbit holes to skip, and which
one has the treasure at the bottom.

---

*Want the gory details? The full research log is in `docs/research/t1778-erf-kernel-research-plan.md`, and every
quirk is catalogued in `docs/excel-semantics/statistics.md` and `numbers.md`.*
