//! Independent aggregator components share one admission rule.
use biomcp_fetch::Terms;
use biomcp_types::{Source, Use};
/// One aggregator component with independent publisher terms.
#[derive(Clone, Copy, Debug)]
pub struct Component {
    /// Aggregator that carries this component.
    pub source: Source,
    /// Explicit native source switch, when this component has one.
    pub native_source: Option<Source>,
    /// Native component name.
    pub name: &'static str,
    /// Component publisher's terms, independent of public API reachability.
    pub terms: Terms,
}
impl Component {
    /// Apply independent publisher policy to an aggregator field.
    pub fn disabled(&self, fetch: &biomcp_fetch::Fetch) -> bool {
        self.policy(
            |source| fetch.disabled(source),
            |usage| fetch.disabled_use(usage),
        )
    }
    /// Apply that same policy inside a deadline-scoped request.
    pub fn disabled_on_leg(&self, leg: &biomcp_fetch::Leg<'_>) -> bool {
        self.policy(
            |source| leg.disabled_source(source),
            |usage| leg.disabled_use(usage),
        )
    }
    pub(crate) fn policy(
        &self,
        source_disabled: impl Fn(Source) -> bool,
        use_disabled: impl Fn(Use) -> bool,
    ) -> bool {
        source_disabled(self.source)
            || self.native_source.is_some_and(source_disabled)
            || use_disabled(self.terms.usage)
    }
}
