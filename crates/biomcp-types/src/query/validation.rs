//! Checked argument identities shared by query constructors and input adapters.
use serde::Deserializer;

/// A query failed a typed input, cross-field rule or bound.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueryInvalid {
    arguments: Box<[Box<str>]>,
    message: Option<crate::ErrorMessage>,
}
impl QueryInvalid {
    /// Retain argument identities only; callers must never pass rejected values.
    pub fn new(arguments: &[&str]) -> Self {
        Self {
            arguments: arguments.iter().map(|name| Box::from(*name)).collect(),
            message: None,
        }
    }
    /// Preserve fixed refusal guidance from the checked constructor.
    pub fn with_message(mut self, message: crate::ErrorMessage) -> Self {
        self.message = Some(message);
        self
    }
    /// Fixed guidance, without rejected input.
    pub fn message(&self) -> Option<crate::ErrorMessage> {
        self.message
    }
    /// Translate a field identity when an adapter renames the input field.
    pub fn rename(mut self, from: &str, to: &str) -> Self {
        for name in &mut self.arguments {
            if name.as_ref() == from {
                *name = to.into();
            }
        }
        self
    }
    /// Identities of the fields involved in the failed rule.
    pub fn arguments(&self) -> &[Box<str>] {
        &self.arguments
    }
}
impl std::fmt::Display for QueryInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("query.invalid")
    }
}
impl std::error::Error for QueryInvalid {}
impl serde::de::Error for QueryInvalid {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self::default()
    }
}

/// Decode fields and preserve the constructor's typed validation cause.
pub trait CheckedQuery: Sized {
    /// Fixed command guidance for failed checked input.
    const USAGE_MESSAGE: Option<crate::ErrorMessage> = None;
    /// Name a rejected field without retaining its value.
    fn invalid_field(name: &str) -> QueryInvalid {
        QueryInvalid::new(&[name])
    }
    /// Decode checked fields once, then call the owning constructor.
    fn from_fields<'de, D: Deserializer<'de, Error = QueryInvalid>>(
        deserializer: D,
    ) -> Result<Self, QueryInvalid>;
}

macro_rules! checked_query {
    ($query:ty, $fields:ty) => {
        $crate::query::validation::checked_query!($query, $fields, None);
    };
    ($query:ty, $fields:ty, $message:expr) => {
        impl $crate::CheckedQuery for $query {
            const USAGE_MESSAGE: Option<$crate::ErrorMessage> = $message;
            fn from_fields<'de, D: serde::Deserializer<'de, Error = $crate::QueryInvalid>>(
                deserializer: D,
            ) -> Result<Self, $crate::QueryInvalid> {
                let fields = <$fields as serde::Deserialize>::deserialize(deserializer)?;
                Self::try_from(fields)
            }
        }
    };
    ($query:ty) => {
        impl $crate::CheckedQuery for $query {
            fn from_fields<'de, D: serde::Deserializer<'de, Error = $crate::QueryInvalid>>(
                deserializer: D,
            ) -> Result<Self, $crate::QueryInvalid> {
                <Self as serde::Deserialize>::deserialize(deserializer)
            }
        }
    };
}
pub(crate) use checked_query;
