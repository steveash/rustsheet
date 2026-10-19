//! `lifting_table_functions!`: the functions of the lifting table (`kinds/`) as a `match`
//! pattern, apart from the table so each file stays a readable size.

/// The functions of [`Function::arg_kinds`](crate::functions::Function::arg_kinds), as a pattern: the arm of an exhaustive `match` on a
/// `Function` that has no hand-written arm because the table derives it (static analysis).
/// Keep it equal to the table; `static_analysis::lifted`'s test checks that no function in the
/// table has a fallback arm.
macro_rules! lifting_table_functions {
    () => {
        Function::Daverage
            | Function::Dcount
            | Function::Dcounta
            | Function::Dget
            | Function::Dmax
            | Function::Dmin
            | Function::Dproduct
            | Function::Dstdev
            | Function::Dstdevp
            | Function::Dsum
            | Function::Dvar
            | Function::Dvarp
            | Function::Edate
            | Function::Eomonth
            | Function::Weeknum
            | Function::Yearfrac
            | Function::Workday
            | Function::WorkdayIntl
            | Function::Networkdays
            | Function::NetworkdaysIntl
            | Function::Isnumber
            | Function::Istext
            | Function::Isnontext
            | Function::Islogical
            | Function::Isblank
            | Function::Iserror
            | Function::Iserr
            | Function::Isna
            | Function::ErrorType
            | Function::Encodeurl
            | Function::Iseven
            | Function::Isodd
            | Function::Large
            | Function::ZTest
            | Function::ZTestCompat
            | Function::Prob
            | Function::Forecast
            | Function::ForecastLinear
            | Function::TTest
            | Function::TTestCompat
            | Function::Small
            | Function::PercentileCompat
            | Function::PercentileInc
            | Function::PercentileExc
            | Function::PercentrankCompat
            | Function::PercentrankInc
            | Function::PercentrankExc
            | Function::QuartileCompat
            | Function::QuartileInc
            | Function::QuartileExc
            | Function::Trimmean
            | Function::Address
            | Function::Sequence
            | Function::Makearray
            | Function::Hyperlink
            | Function::Pmt
            | Function::Pv
            | Function::Fv
            | Function::Nper
            | Function::Rate
            | Function::Ipmt
            | Function::Ppmt
            | Function::Ispmt
            | Function::Rri
            | Function::Pduration
            | Function::Sln
            | Function::Syd
            | Function::Ddb
            | Function::Effect
            | Function::Nominal
            | Function::Dollarde
            | Function::Dollarfr
            | Function::Accrintm
            | Function::Disc
            | Function::Intrate
            | Function::Pricedisc
            | Function::Received
            | Function::Yielddisc
            | Function::Pricemat
            | Function::Yieldmat
            | Function::Tbilleq
            | Function::Tbillprice
            | Function::Tbillyield
            // --- Engineering (T-1383) ---
            | Function::Besseli
            | Function::Besselj
            | Function::Besselk
            | Function::Bessely
            | Function::Erf
            | Function::ErfPrecise
            | Function::Erfc
            | Function::ErfcPrecise
            | Function::Bin2dec
            | Function::Bin2hex
            | Function::Bin2oct
            | Function::Dec2Bin
            | Function::Dec2hex
            | Function::Dec2oct
            | Function::Hex2bin
            | Function::Hex2dec
            | Function::Hex2oct
            | Function::Oct2bin
            | Function::Oct2dec
            | Function::Oct2hex
            | Function::Bitand
            | Function::Bitor
            | Function::Bitxor
            | Function::Bitlshift
            | Function::Bitrshift
            | Function::Complex
            | Function::Convert
            | Function::Delta
            | Function::Gestep
            | Function::Imabs
            | Function::Imaginary
            | Function::Imargument
            | Function::Imconjugate
            | Function::Imcos
            | Function::Imcosh
            | Function::Imcot
            | Function::Imcsc
            | Function::Imcsch
            | Function::Imdiv
            | Function::Imexp
            | Function::Imln
            | Function::Imlog10
            | Function::Imlog2
            | Function::Impower
            | Function::Imreal
            | Function::Imsec
            | Function::Imsech
            | Function::Imsin
            | Function::Imsinh
            | Function::Imsqrt
            | Function::Imsub
            | Function::Imtan
            | Function::Imsum
            | Function::Improduct
            // --- End engineering (T-1383) ---
            // --- Math (T-1802) ---
            | Function::Quotient
            | Function::Factdouble
            | Function::Seriessum
            // --- End math (T-1802) ---
            // --- Math number lifts (T-1773) ---
            // (alphabetical)
            | Function::Abs
            | Function::Acos
            | Function::Acosh
            | Function::Acot
            | Function::Acoth
            | Function::Asin
            | Function::Asinh
            | Function::Atan
            | Function::Atanh
            | Function::Ceiling
            | Function::CeilingMath
            | Function::CeilingPrecise
            | Function::Cos
            | Function::Cosh
            | Function::Cot
            | Function::Coth
            | Function::Csc
            | Function::Csch
            | Function::Degrees
            | Function::EcmaCeiling
            | Function::Even
            | Function::Exp
            | Function::Fact
            | Function::Floor
            | Function::FloorMath
            | Function::FloorPrecise
            | Function::Int
            | Function::IsoCeiling
            | Function::Ln
            | Function::Log10
            | Function::Mod
            | Function::Odd
            | Function::Radians
            | Function::Round
            | Function::Rounddown
            | Function::Roundup
            | Function::Sec
            | Function::Sech
            | Function::Sign
            | Function::Sin
            | Function::Sinh
            | Function::Sqrt
            | Function::Tan
            | Function::Tanh
            | Function::Trunc
            // --- End math number lifts (T-1773) ---
            // --- Financial ATP (T-1822) ---
            | Function::Accrint
            | Function::Coupdaybs
            | Function::Coupdays
            | Function::Coupdaysnc
            | Function::Coupncd
            | Function::Coupnum
            | Function::Couppcd
            | Function::Duration
            | Function::Mduration
            | Function::Price
            // --- End financial ATP (T-1822) ---
            // --- Financial ATP (T-1842) ---
            | Function::Yield
            | Function::Oddfprice
            | Function::Oddfyield
            | Function::Oddlprice
            | Function::Oddlyield
            // --- End financial ATP (T-1842) ---
            // --- Financial ATP (T-1845) ---
            | Function::Cumipmt
            | Function::Cumprinc
            // --- End financial ATP (T-1845) ---
            // --- Financial ATP (T-1841) ---
            | Function::Amorlinc
            | Function::Amordegrc
            // --- End financial ATP (T-1841) ---
            // --- Statistical (T-1290) ---
            | Function::BetaDist
            | Function::BetaDistCompat
            | Function::BinomDist
            | Function::BinomDistCompat
            | Function::BinomDistRange
            | Function::BinomInv
            | Function::CritbinomCompat
            | Function::NegbinomDist
            | Function::NegbinomDistCompat
            | Function::ConfidenceNorm
            | Function::ConfidenceCompat
            | Function::ExponDist
            | Function::ExponDistCompat
            | Function::FDist
            | Function::FDistRT
            | Function::FDistCompat
            | Function::Fisher
            | Function::FisherInv
            | Function::Gamma
            | Function::GammaLn
            | Function::GammaLnPrecise
            | Function::Gauss
            | Function::Phi
            | Function::LogNormDist
            | Function::LogNormDistCompat
            | Function::LogNormInv
            | Function::LoginvCompat
            | Function::NormDist
            | Function::NormDistCompat
            | Function::NormInv
            | Function::NormInvCompat
            | Function::NormSdist
            | Function::NormSDistCompat
            | Function::NormSInv
            | Function::NormSInvCompat
            | Function::Permut
            | Function::Permutationa
            | Function::Standardize
            | Function::TDist
            | Function::TDist2T
            | Function::TDistRT
            | Function::TDistCompat
            | Function::WeibullDist
            | Function::WeibullCompat
            // --- End statistical (T-1290) ---
            // --- Statistical quantiles (T-1848) ---
            | Function::BetaInv
            | Function::BetaInvCompat
            | Function::FInv
            | Function::FInvRT
            | Function::FInvCompat
            | Function::TInv
            | Function::TInv2T
            | Function::TInvCompat
            | Function::ConfidenceT
            // --- End statistical quantiles (T-1848) ---
            // --- Incomplete gamma (T-1872) ---
            | Function::ChisqDist
            | Function::ChisqDistRT
            | Function::ChiDistCompat
            | Function::ChisqInv
            | Function::ChisqInvRT
            | Function::ChiInvCompat
            | Function::GammaDist
            | Function::GammaDistCompat
            | Function::GammaInv
            | Function::GammaInvCompat
            | Function::PoissonDist
            | Function::PoissonCompat
            // --- End incomplete gamma (T-1872) ---
            // --- Hypergeometric (T-1873) ---
            | Function::HypGeomDist
            | Function::HypGeomDistCompat
            // --- End hypergeometric (T-1873) ---
            // --- Date and time lifts, MROUND, SQRTPI (T-1785) ---
            | Function::Day
            | Function::Month
            | Function::Year
            | Function::Hour
            | Function::Minute
            | Function::Second
            | Function::Isoweeknum
            | Function::Weekday
            | Function::Days
            | Function::Days360
            | Function::Date
            | Function::Time
            | Function::Datedif
            | Function::Mround
            | Function::Sqrtpi
            // --- End date and time lifts, MROUND, SQRTPI (T-1785) ---
    };
}
pub(crate) use lifting_table_functions;
