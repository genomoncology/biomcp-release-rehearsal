//! Concrete response pages retain their public schema names and field order.
macro_rules! page {
    (protein, $name:ident, $row:ty, $query:ty, $schema:literal, $doc:literal) => {
        page!(
            @body, $name, $row, $query, $schema, $doc, [], "Whole native rows in source order; totals apply only to this collection.", [], []);
    };
    (diagnostic, $name:ident, $row:ty, $query:ty, $schema:literal, $doc:literal) => {
        page!(
            @body, $name, $row, $query, $schema, $doc, [], "Complete source-owned rows in native order; envelope totals count native rows.", [], [
            /// Exact meaning of the envelope total; supplied by the owning source command.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub count_unit: Option<DiagnosticCountUnit>,
            /// Distinct complete native IDs across the selected relation collection.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub distinct_id_total: Option<u64>,]);
    };
    (study, $name:ident, $row:ty, $query:ty, $schema:literal, $doc:literal) => {
        page!(
            @body, $name, $row, $query, $schema, $doc, [Eq], "Complete ordered metadata records, never member sample IDs.", [
            /// Exact study scope of this collection.
            pub study_id: StudyId,], [
            /// Bounds and source-stated total for this collection.
            pub page: PageInfo,
            /// Live collection acquisition wall time in Unix seconds, distinct from catalog acquisition.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub acquired: Option<u64>,]);
    };
    (disease, $name:ident, $row:ty, $query:ty, $schema:literal, $doc:literal $(, [$($extra:tt)*])?) => {
        page!(
            @body, $name, $row, $query, $schema, $doc, [], "Native source rows; totals in the envelope count rows, not distinct entities.", [], [
            /// Applicable component use terms, independent of row provenance.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub components: Box<[ComponentLabel]>,
            $($($extra)*)?]);
    };
    (pathway, $name:ident, $row:ty, $query:ty, $schema:literal, $doc:literal) => {
        page!(
            @body, $name, $row, $query, $schema, $doc, [], "Whole native references; unresolved identities remain visible.", [
            /// Exact source pathway owning this relation.
            pub header: PathwayHeader,], [
            /// Relation-specific count provenance and continuation.
            pub page: PathwayPageInfo,]);
    };
    (phenotype, $name:ident, $row:ty, $query:ty, $schema:literal, $doc:literal) => {
        page!(
            @body, $name, $row, $query, $schema, $doc, [], "Native ordered rows; empty remains an empty collection.", [], [
            /// Applicable publisher component terms.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub components: Box<[ComponentLabel]>,
            /// Original source retrieval seconds, distinct from ontology release.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub fetched: Option<u64>,]);
    };
    (@body, $name:ident, $row:ty, $query:ty, $schema:literal, $doc:literal,
     [$($extra_derive:ident),*], $items_doc:literal, [$($before:tt)*], [$($after:tt)*]) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, $($extra_derive,)* serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name {
            $($before)*
            #[doc = $items_doc]
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub items: Box<[$row]>,
            $($after)*
        }
        impl $crate::ResponseBody for $name {
            const SCHEMA: &'static str = $schema;
            type Query = $query;
        }
    };
}
