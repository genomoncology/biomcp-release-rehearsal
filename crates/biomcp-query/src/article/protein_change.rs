//! Reader-only literal matching over the shared checked substitution value.
use biodata::Text;
#[derive(Debug)]
pub(super) struct Selection {
    pub(super) forms: Option<[String; 4]>,
    pub(super) needles: Vec<String>,
}
impl Selection {
    pub(super) fn new(
        find: Option<&Text>,
        variant: Option<&Text>,
    ) -> Result<Self, biomcp_types::QueryInvalid> {
        let forms = variant
            .map(|v| {
                biomcp_types::protein_query_forms(v.as_str())
                    .map_err(|_| biomcp_types::QueryInvalid::default())
            })
            .transpose()?;
        let needles = forms.clone().map(|v| v.to_vec()).unwrap_or_else(|| {
            find.map(|v| vec![v.as_str().to_owned()])
                .unwrap_or_default()
        });
        Ok(Self { forms, needles })
    }
}
