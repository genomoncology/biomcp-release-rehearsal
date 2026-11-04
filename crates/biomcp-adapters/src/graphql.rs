//! Ordered GraphQL alias batches preserve duplicate keys for admission checks.
use biomcp_fetch::BadReply;
use serde::{
    Deserialize, Serialize,
    de::{MapAccess, Visitor},
};
use std::collections::BTreeMap;
#[derive(Serialize)]
pub(crate) struct Query {
    pub(crate) query: String,
    pub(crate) variables: BTreeMap<String, String>,
}
impl Query {
    pub(crate) fn aliases(
        values: impl Iterator<Item = String>,
        field: impl Fn(usize) -> String,
    ) -> Self {
        let mut variables = BTreeMap::new();
        let mut declarations = Vec::new();
        let mut aliases = Vec::new();
        for (index, value) in values.enumerate() {
            variables.insert(format!("id{index}"), value);
            declarations.push(format!("$id{index}: String!"));
            aliases.push(format!("g{index}: {}", field(index)));
        }
        Self {
            query: format!(
                "query({}) {{ {} }}",
                declarations.join(", "),
                aliases.join(" ")
            ),
            variables,
        }
    }
}
#[derive(Debug)]
pub(crate) struct Aliases<T>(pub(crate) Vec<(String, Option<T>)>);
impl<T> Aliases<T> {
    pub(crate) fn check<const N: usize>(&self) -> Result<(), BadReply> {
        if self.0.len() != N
            || (0..N).any(|index| {
                self.0
                    .iter()
                    .filter(|(name, _)| *name == format!("g{index}"))
                    .count()
                    != 1
            })
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Aliases<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Map<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for Map<T> {
            type Value = Aliases<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("gene aliases")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Aliases<T>, A::Error> {
                let mut aliases = Vec::new();
                while let Some(pair) = map.next_entry()? {
                    aliases.push(pair);
                }
                Ok(Aliases(aliases))
            }
        }
        d.deserialize_map(Map(std::marker::PhantomData))
    }
}
