//! Carry each reply with the query type declared by its response body.
use crate::{ReplyData, ResponseBody, UsageError};

/// A registered body can retain its associated query until visitation.
///
/// ```compile_fail
/// use biomcp_types::{GeneGet, GenePage, ReplyBody, ReplyData};
/// fn wrong_pair(query: GeneGet, body: GenePage) -> ReplyData {
///     GenePage::into_reply(query, Some(body))
/// }
/// ```
pub trait ReplyBody: ResponseBody + Sized {
    /// Retain one type-correct reply pair.
    fn into_reply(query: Self::Query, body: Option<Self>) -> ReplyData;
    /// Retain a present body with its associated checked query.
    fn with_query(self, query: Self::Query) -> ReplyData {
        Self::into_reply(query, Some(self))
    }
}

macro_rules! replies {
    ($($method:ident: $body:ty => $name:literal),+ $(,)?) => {
        /// A visitor receives each body with its associated checked query type.
        pub trait Reply: Sized {
            /// Result selected by the interface.
            type Output;
            /// Construct one typed reply.
            fn reply<B: ReplyBody>(self, query: B::Query, body: Option<B>) -> Self::Output;
            /// Refuse a query before producing a body.
            fn usage(self, error: UsageError) -> Self::Output;
            /// Spell snapshot recovery before selecting usage or expiration.
            fn pool(self, mismatch: bool) -> Self::Output;
            $(#[doc = concat!("Visit ", $name, ".")]
            fn $method(self, query: <$body as ResponseBody>::Query, body: Option<$body>) -> Self::Output {
                self.reply::<$body>(query, body)
            })+
        }
    };
}
crate::reply_types!(replies);
