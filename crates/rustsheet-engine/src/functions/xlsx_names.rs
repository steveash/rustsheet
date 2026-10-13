//! Function names as Excel writes them to xlsx: newer functions carry the `_xlfn.` (and
//! `_xlfn._xlws.`) prefixes. Split out of `functions/mod.rs` to keep that file under its
//! size ratchet. `tools/src/rustsheet_tools/spec2xlsx.py` reads the prefixed arms of this file with a
//! regex, so keep each prefixed name on its own match arm.

use super::Function;
use crate::language::get_default_language;

impl Function {
    /// Some functions in Excel like CONCAT are stringified as `_xlfn.CONCAT`. (known as "future functions")
    /// FILTER and SORT are stringified as `_xlfn._xlws.FILTER`. (known as "worksheet only functions")
    /// See <https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/3d025add-118d-4413-9856-ab65712ec1b0>
    pub fn to_xlsx_string(&self) -> String {
        match self {
            Function::Aggregate => "_xlfn.AGGREGATE".to_string(),
            Function::Concat => "_xlfn.CONCAT".to_string(),
            Function::Ifna => "_xlfn.IFNA".to_string(),
            Function::Ifs => "_xlfn.IFS".to_string(),
            Function::Lambda => "_xlfn.LAMBDA".to_string(),
            Function::Let => "_xlfn.LET".to_string(),
            Function::Maxifs => "_xlfn.MAXIFS".to_string(),
            Function::Minifs => "_xlfn.MINIFS".to_string(),
            Function::Switch => "_xlfn.SWITCH".to_string(),
            Function::Filter => "_xlfn._xlws.FILTER".to_string(),
            Function::Sort => "_xlfn._xlws.SORT".to_string(),
            Function::Sortby => "_xlfn.SORTBY".to_string(),
            Function::Unique => "_xlfn.UNIQUE".to_string(),
            Function::Xlookup => "_xlfn.XLOOKUP".to_string(),
            Function::Xmatch => "_xlfn.XMATCH".to_string(),
            Function::Trimrange => "_xlfn.TRIMRANGE".to_string(),
            // Stored bare it is #NAME? in Excel (probes/catalog/function-name-forms A1/A2, T-0416).
            Function::Percentof => "_xlfn.PERCENTOF".to_string(),
            Function::Take => "_xlfn.TAKE".to_string(),
            Function::Drop => "_xlfn.DROP".to_string(),
            Function::Tocol => "_xlfn.TOCOL".to_string(),
            Function::Torow => "_xlfn.TOROW".to_string(),
            Function::Choosecols => "_xlfn.CHOOSECOLS".to_string(),
            Function::Chooserows => "_xlfn.CHOOSEROWS".to_string(),
            Function::Expand => "_xlfn.EXPAND".to_string(),
            Function::Hstack => "_xlfn.HSTACK".to_string(),
            Function::Vstack => "_xlfn.VSTACK".to_string(),
            Function::Wrapcols => "_xlfn.WRAPCOLS".to_string(),
            Function::Wraprows => "_xlfn.WRAPROWS".to_string(),
            Function::Bycol => "_xlfn.BYCOL".to_string(),
            Function::Byrow => "_xlfn.BYROW".to_string(),
            Function::Map => "_xlfn.MAP".to_string(),
            Function::Reduce => "_xlfn.REDUCE".to_string(),
            Function::Scan => "_xlfn.SCAN".to_string(),
            Function::Makearray => "_xlfn.MAKEARRAY".to_string(),
            Function::Xor => "_xlfn.XOR".to_string(),
            Function::Textbefore => "_xlfn.TEXTBEFORE".to_string(),
            Function::Textafter => "_xlfn.TEXTAFTER".to_string(),
            Function::Textsplit => "_xlfn.TEXTSPLIT".to_string(),
            Function::Textjoin => "_xlfn.TEXTJOIN".to_string(),
            Function::Sequence => "_xlfn.SEQUENCE".to_string(),
            Function::Randarray => "_xlfn.RANDARRAY".to_string(),
            Function::Unicode => "_xlfn.UNICODE".to_string(),
            Function::Unichar => "_xlfn.UNICHAR".to_string(),
            Function::Numbervalue => "_xlfn.NUMBERVALUE".to_string(),
            Function::Arraytotext => "_xlfn.ARRAYTOTEXT".to_string(),
            Function::Dbcs => "_xlfn.DBCS".to_string(),
            Function::Encodeurl => "_xlfn.ENCODEURL".to_string(),
            Function::Image => "_xlfn.IMAGE".to_string(),
            Function::Stockhistory => "_xlfn.STOCKHISTORY".to_string(),
            Function::Fieldvalue => "_xlfn.FIELDVALUE".to_string(),
            Function::Rri => "_xlfn.RRI".to_string(),
            Function::Pduration => "_xlfn.PDURATION".to_string(),
            Function::Bitand => "_xlfn.BITAND".to_string(),
            Function::Bitor => "_xlfn.BITOR".to_string(),
            Function::Bitxor => "_xlfn.BITXOR".to_string(),
            Function::Bitlshift => "_xlfn.BITLSHIFT".to_string(),
            Function::Bitrshift => "_xlfn.BITRSHIFT".to_string(),
            Function::Imtan => "_xlfn.IMTAN".to_string(),
            Function::Imsinh => "_xlfn.IMSINH".to_string(),
            Function::Imcosh => "_xlfn.IMCOSH".to_string(),
            Function::Imcot => "_xlfn.IMCOT".to_string(),
            Function::Imcsc => "_xlfn.IMCSC".to_string(),
            Function::Imcsch => "_xlfn.IMCSCH".to_string(),
            Function::Imsec => "_xlfn.IMSEC".to_string(),
            Function::ErfcPrecise => "_xlfn.ERFC.PRECISE".to_string(),
            Function::ErfPrecise => "_xlfn.ERF.PRECISE".to_string(),
            Function::Valuetotext => "_xlfn.VALUETOTEXT".to_string(),
            Function::Regexextract => "_xlfn.REGEXEXTRACT".to_string(),
            Function::Regexreplace => "_xlfn.REGEXREPLACE".to_string(),
            Function::Regextest => "_xlfn.REGEXTEST".to_string(),
            Function::Isformula => "_xlfn.ISFORMULA".to_string(),
            Function::Isomitted => "_xlfn.ISOMITTED".to_string(),
            Function::Sheet => "_xlfn.SHEET".to_string(),
            Function::Formulatext => "_xlfn.FORMULATEXT".to_string(),
            Function::Isoweeknum => "_xlfn.ISOWEEKNUM".to_string(),
            Function::Days => "_xlfn.DAYS".to_string(),
            Function::Gauss => "_xlfn.GAUSS".to_string(),
            Function::Imsech => "_xlfn.IMSECH".to_string(),
            Function::GammaDist => "_xlfn.GAMMA.DIST".to_string(),
            Function::ForecastLinear => "_xlfn.FORECAST.LINEAR".to_string(),
            Function::CeilingMath => "_xlfn.CEILING.MATH".to_string(),
            Function::CeilingPrecise => "_xlfn.CEILING.PRECISE".to_string(),
            Function::FloorMath => "_xlfn.FLOOR.MATH".to_string(),
            Function::FloorPrecise => "_xlfn.FLOOR.PRECISE".to_string(),
            Function::IsoCeiling => "_xlfn.ISO.CEILING".to_string(),
            Function::EcmaCeiling => "_xlfn.ECMA.CEILING".to_string(),
            Function::Base => "_xlfn.BASE".to_string(),
            Function::Decimal => "_xlfn.DECIMAL".to_string(),
            Function::Arabic => "_xlfn.ARABIC".to_string(),
            Function::Combina => "_xlfn.COMBINA".to_string(),
            Function::Sheets => "_xlfn.SHEETS".to_string(),
            Function::Acoth => "_xlfn.ACOTH".to_string(),
            Function::Cot => "_xlfn.COT".to_string(),
            Function::Coth => "_xlfn.COTH".to_string(),
            Function::Csc => "_xlfn.CSC".to_string(),
            Function::Csch => "_xlfn.CSCH".to_string(),
            Function::Sec => "_xlfn.SEC".to_string(),
            Function::Sech => "_xlfn.SECH".to_string(),
            Function::Acot => "_xlfn.ACOT".to_string(),
            Function::GammaLnPrecise => "_xlfn.GAMMALN.PRECISE".to_string(),
            Function::Gamma => "_xlfn.GAMMA".to_string(),
            Function::GammaInv => "_xlfn.GAMMA.INV".to_string(),
            Function::GammaLn => "_xlfn.GAMMALN".to_string(),
            Function::BetaDist => "_xlfn.BETA.DIST".to_string(),
            Function::BetaInv => "_xlfn.BETA.INV".to_string(),

            Function::BinomDist => "_xlfn.BINOM.DIST".to_string(),
            Function::BinomDistRange => "_xlfn.BINOM.DIST.RANGE".to_string(),
            Function::BinomInv => "_xlfn.BINOM.INV".to_string(),
            Function::NegbinomDist => "_xlfn.NEGBINOM.DIST".to_string(),

            Function::ChisqDist => "_xlfn.CHISQ.DIST".to_string(),
            Function::ChisqDistRT => "_xlfn.CHISQ.DIST.RT".to_string(),
            Function::ChisqInv => "_xlfn.CHISQ.INV".to_string(),
            Function::ChisqInvRT => "_xlfn.CHISQ.INV.RT".to_string(),
            Function::ChisqTest => "_xlfn.CHISQ.TEST".to_string(),

            Function::ConfidenceNorm => "_xlfn.CONFIDENCE.NORM".to_string(),
            Function::ConfidenceT => "_xlfn.CONFIDENCE.T".to_string(),

            Function::CovarianceP => "_xlfn.COVARIANCE.P".to_string(),
            Function::CovarianceS => "_xlfn.COVARIANCE.S".to_string(),

            Function::ExponDist => "_xlfn.EXPON.DIST".to_string(),

            Function::FDist => "_xlfn.F.DIST".to_string(),
            Function::FDistRT => "_xlfn.F.DIST.RT".to_string(),
            Function::FInv => "_xlfn.F.INV".to_string(),
            Function::FInvRT => "_xlfn.F.INV.RT".to_string(),
            Function::FTest => "_xlfn.F.TEST".to_string(),

            Function::HypGeomDist => "_xlfn.HYPGEOM.DIST".to_string(),

            Function::LogNormDist => "_xlfn.LOGNORM.DIST".to_string(),
            Function::LogNormInv => "_xlfn.LOGNORM.INV".to_string(),

            Function::NormDist => "_xlfn.NORM.DIST".to_string(),
            Function::NormInv => "_xlfn.NORM.INV".to_string(),
            Function::NormSdist => "_xlfn.NORM.S.DIST".to_string(),
            Function::NormSInv => "_xlfn.NORM.S.INV".to_string(),

            Function::Phi => "_xlfn.PHI".to_string(),

            Function::PoissonDist => "_xlfn.POISSON.DIST".to_string(),

            Function::StDevP => "_xlfn.STDEV.P".to_string(),
            Function::StDevS => "_xlfn.STDEV.S".to_string(),

            Function::TDist => "_xlfn.T.DIST".to_string(),
            Function::TDist2T => "_xlfn.T.DIST.2T".to_string(),
            Function::TDistRT => "_xlfn.T.DIST.RT".to_string(),
            Function::TInv => "_xlfn.T.INV".to_string(),
            Function::TInv2T => "_xlfn.T.INV.2T".to_string(),
            Function::TTest => "_xlfn.T.TEST".to_string(),

            Function::VarP => "_xlfn.VAR.P".to_string(),
            Function::VarS => "_xlfn.VAR.S".to_string(),

            Function::WeibullDist => "_xlfn.WEIBULL.DIST".to_string(),
            Function::ZTest => "_xlfn.Z.TEST".to_string(),
            Function::SkewP => "_xlfn.SKEW.P".to_string(),
            Function::RankAvg => "_xlfn.RANK.AVG".to_string(),
            Function::RankEq => "_xlfn.RANK.EQ".to_string(),
            Function::Munit => "_xlfn.MUNIT".to_string(),

            Function::ModeMult => "_xlfn.MODE.MULT".to_string(),
            Function::ModeSingl => "_xlfn.MODE.SNGL".to_string(),
            Function::PercentileExc => "_xlfn.PERCENTILE.EXC".to_string(),
            Function::PercentileInc => "_xlfn.PERCENTILE.INC".to_string(),
            Function::PercentrankExc => "_xlfn.PERCENTRANK.EXC".to_string(),
            Function::PercentrankInc => "_xlfn.PERCENTRANK.INC".to_string(),
            Function::Permutationa => "_xlfn.PERMUTATIONA".to_string(),
            Function::QuartileExc => "_xlfn.QUARTILE.EXC".to_string(),
            Function::QuartileInc => "_xlfn.QUARTILE.INC".to_string(),

            Function::ForecastEts => "_xlfn.FORECAST.ETS".to_string(),
            Function::ForecastEtsConfint => "_xlfn.FORECAST.ETS.CONFINT".to_string(),
            Function::ForecastEtsSeasonality => "_xlfn.FORECAST.ETS.SEASONALITY".to_string(),
            Function::ForecastEtsStat => "_xlfn.FORECAST.ETS.STAT".to_string(),

            _ => {
                let language = get_default_language();
                self.to_localized_name(language)
            }
        }
    }
}
