//! Shared reviewed policy for both pharmacogenomics publishers.
use biomcp_fetch::Terms;
use biomcp_types::Use;
// The captured policy page was a shell; keep its access restriction explicit.
pub(crate) const TERMS: Terms = Terms {
    name: "ClinPGx data use policy; noncommercial access restriction; captured policy was a JavaScript shell",
    url: "https://www.clinpgx.org/page/dataUsagePolicy",
    usage: Use::Noncommercial,
};

/// The two native PGx publishers share field mapping, not admission policy.
pub(crate) trait Native {
    type Value;
    fn record(&self) -> Option<biodata::Text>;
    fn map(
        self,
        problems: &mut crate::mapped::Problems,
        refusals: &mut crate::Refusal,
    ) -> Option<Self::Value>;
}

macro_rules! pgx_row {
    ($name:ident => $out:path, record $record:ident {
        identity { $($(#[$ia:meta])* $id:ident : $it:ty => $ip:literal $(as $authority:literal)?,)* }
        values { $($(#[$va:meta])* $value:ident : $vt:ty => $vp:literal $(as $value_authority:literal)?,)* }
        nested { $($(#[$na:meta])* $nested:ident : $nt:ty => $np:literal,)* }
        lists { $($(#[$la:meta])* $list:ident : $lt:ty => $lp:literal $(as $list_authority:literal)?,)* }
        nested_lists { $($(#[$ma:meta])* $members:ident : $mt:ty => $mp:literal,)* }
        optional_lists { $($(#[$oa:meta])* $optional:ident : $ot:ty => $op:literal,)* }
    }) => {
        #[derive(serde::Deserialize)]
        pub(super) struct $name {
            $($(#[$ia])* $id: Option<crate::raw::Field<$it>>,)*
            $($(#[$va])* $value: Option<crate::raw::Field<$vt>>,)*
            $($(#[$na])* $nested: Option<crate::raw::Field<$nt>>,)*
            $($(#[$la])* $list: Option<crate::raw::Field<Vec<crate::raw::Field<$lt>>>>,)*
            $($(#[$ma])* $members: Option<crate::raw::Field<Vec<crate::raw::Field<$mt>>>>,)*
            $($(#[$oa])* $optional: Option<crate::raw::Field<Vec<crate::raw::Field<$ot>>>>,)*
        }
        impl crate::pgx_sources::Native for $name {
            type Value = $out;
            fn record(&self) -> Option<biodata::Text> {
                self.$record.as_ref().and_then(crate::raw::Field::as_ref)
                    .and_then(crate::mapped::record)
            }
            fn map(
                self,
                problems: &mut crate::mapped::Problems,
                refusals: &mut crate::Refusal,
            ) -> Option<Self::Value> {
                let mapped = (|| {
                $(let $id = problems.required_field($ip, self.$id)?;)*
                problems.text(stringify!($record), Some(&$record.to_string()))?;
                $(let $id = pgx_row!(@identity problems, $ip, $id $(, $authority)?);)*
                use $out as Output;
                Some(Output {
                    $($id,)*
                    $($value: pgx_row!(@value problems, $vp, self.$value $(, $value_authority)?),)*
                    $($nested: self.$nested
                        .and_then(|row| problems.record($np, row, refusals))
                        .and_then(|row| row.map(problems, refusals)),)*
                    $($list: pgx_row!(@list problems, $lp, self.$list $(, $list_authority)?),)*
                    $($members: problems.records($mp, self.$members, refusals).into_vec().into_iter()
                        .filter_map(|row| row.map(problems, refusals)).collect(),)*
                    $($optional: problems.field($op, self.$optional).map(|values| values.into_iter()
                        .filter_map(|value| problems.field($op, Some(value))).collect()),)*
                })
                })();
                if mapped.is_none() {
                    *refusals = crate::Refusal::Refused;
                }
                mapped
            }
        }
    };
    (@identity $problems:ident, $path:literal, $value:ident, $authority:literal) => {
        $problems.optional($path, Some(crate::mapped::identifier($authority, $value)))?
    };
    (@identity $problems:ident, $path:literal, $value:ident) => { $value };
    (@value $problems:ident, $path:literal, $field:expr, $authority:literal) => {
        $problems.field($path, $field).and_then(|id| {
            $problems.optional($path, Some(crate::mapped::identifier($authority, id)))
        })
    };
    (@value $problems:ident, $path:literal, $field:expr) => { $problems.field($path, $field) };
    (@list $problems:ident, $path:literal, $field:expr, $authority:literal) => {
        $problems.fields($path, $field).into_vec().into_iter().filter_map(|id| {
            $problems.optional($path, Some(crate::mapped::identifier($authority, id)))
        }).collect()
    };
    (@list $problems:ident, $path:literal, $field:expr) => { $problems.fields($path, $field) };
}
pub(crate) use pgx_row;
