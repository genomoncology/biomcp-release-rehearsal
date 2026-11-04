//! Retain typed query and body pairs until an interface visits the reply.
use crate::{Reply, ReplyBody, ResponseBody, UsageError};

macro_rules! packets {
    ($($method:ident: $root:ident :: $body:ident => $name:literal),+ $(,)?) => {
        /// A reply whose query cannot differ from its body's associated query type.
        #[derive(Clone, Debug)]
        pub enum ReplyData {
            $(#[doc = concat!("Checked ", $name, " reply.")]
            $body { #[doc = "Checked query."] query: <crate::$body as ResponseBody>::Query,
                #[doc = "Absent only after failed retrieval."] body: Option<crate::$body> },)+
            /// A checked refusal before reply construction.
            Usage(UsageError),
            /// Snapshot recovery whose status is decided after restart spelling.
            PoolError { #[doc = "Whether checked selectors differ."] mismatch: bool },
        }
        $(impl ReplyBody for crate::$body {
            fn into_reply(query: Self::Query, body: Option<Self>) -> ReplyData {
                ReplyData::$body { query, body }
            }
        })+
        impl ReplyData {
            /// Visit one type-correct pair without matching a request separately.
            pub fn visit<R: Reply>(self, visitor: R) -> R::Output {
                match self {
                    $(Self::$body { query, body } => visitor.$method(query, body),)+
                    Self::Usage(error) => visitor.usage(error),
                    Self::PoolError { mismatch } => visitor.pool(mismatch),
                }
            }
        }
    };
}
crate::reply_types!(packets);
