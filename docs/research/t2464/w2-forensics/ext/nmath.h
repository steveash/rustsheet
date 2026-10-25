#include <math.h>
#include <float.h>
#include <stdio.h>
#include <stdbool.h>
typedef int Rboolean;
#define TRUE 1
#define FALSE 0
#define R_D__0 (log_p ? -INFINITY : 0.)
#define R_D__1 (log_p ? 0. : 1.)
#define R_D_exp(x) (log_p ? (x) : exp(x))
#define ML_NEGINF (-INFINITY)
#define ML_POSINF (INFINITY)
#define ML_NAN (NAN)
#define ML_WARN_return_NAN return NAN
#define MATHLIB_WARNING(f,a)
#define MATHLIB_WARNING2(f,a,b)
#define MATHLIB_WARNING3(f,a,b,c)
#define MATHLIB_WARNING4(f,a,b,c,d)
#define MATHLIB_WARNING5(f,a,b,c,d,e)
#define MATHLIB_WARNING6(f,a,b,c,d,e,g)

#define R_FINITE(x) isfinite(x)
#define ISNAN(x) isnan(x)
#define attribute_hidden
static double Rf_d1mach(int i){ switch(i){case 1:return DBL_MIN;case 2:return DBL_MAX;case 3:return DBL_EPSILON/2;case 4:return DBL_EPSILON;case 5:return M_LOG10E*0+0.301029995663981195;} return 0;}
static int Rf_i1mach(int i){ switch(i){case 15:return DBL_MIN_EXP;case 16:return DBL_MAX_EXP;} return 0;}
#define M_LN_SQRT_2PI 0.918938533204672741780329736406
#define M_LN_SQRT_PI 0.572364942924700087071713675677
#define M_SQRT_PI 1.772453850905516027298167483341
#define M_1_SQRT_2PI 0.398942280401432677939946059934
#define M_SQRT_2dPI 0.797884560802865355879892119869
#define M_LN_SQRT_PId2 0.225791352644727432363097614947
static double logspace_add(double lx, double ly){ return fmax(lx,ly) + log1p(exp(-fabs(lx-ly))); }
