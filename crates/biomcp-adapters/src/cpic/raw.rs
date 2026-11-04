//! Native fields retain source spellings and independent error boundaries.
// Native keys and applicability joins follow the publisher's schema and views.
// https://github.com/cpicpgx/cpic-data/tree/main/src/main/resources/db/migration
use crate::pgx_sources::pgx_row;
use biodata::{
    ClinPgxAccession, Curie, EnsemblGeneId, GeneSymbol, HgncId, NcbiGeneId, PgxGeneKey, Pmid,
};
use std::collections::BTreeMap;
pgx_row! { PgxGeneIdentity => biodata::PgxGeneIdentity, record symbol {
    identity {
        symbol: GeneSymbol => "symbol",
    }
    values {
        hgncid: HgncId => "hgncid",
        ncbiid: NcbiGeneId => "ncbiid",
        ensemblid: EnsemblGeneId => "ensemblid",
        clinpgxid: ClinPgxAccession => "clinpgxid",
        lookupmethod: Box<str> => "lookupmethod",
        version: i64 => "version",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
    }
}}
pgx_row! { PgxPair => biodata::PgxPair, record pairid {
    identity {
        pairid: i64 => "pairid" as "CPIC",
        genesymbol: GeneSymbol => "genesymbol",
    }
    values {
        drugid: Curie => "drugid",
        drugname: Box<str> => "drugname",
        guidelinename: Box<str> => "guidelinename",
        guidelineurl: Box<str> => "guidelineurl",
        cpiclevel: Box<str> => "cpiclevel",
        clinpgxlevel: Box<str> => "clinpgxlevel",
        pgxtesting: Box<str> => "pgxtesting",
        usedforrecommendation: Box<str> => "usedforrecommendation",
        provisional: bool => "provisional",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
        pmids: Pmid => "pmids",
    }
}}
pgx_row! { PgxRecommendation => biodata::PgxRecommendation, record recommendationid {
    identity {
        recommendationid: i64 => "recommendationid" as "CPIC",
        lookupkey: BTreeMap<PgxGeneKey, Box<str>> => "lookupkey",
    }
    values {
        drugname: Box<str> => "drugname",
        guidelinename: Box<str> => "guidelinename",
        guidelineurl: Box<str> => "guidelineurl",
        implications: BTreeMap<PgxGeneKey, Box<str>> => "implications",
        drugrecommendation: Box<str> => "drugrecommendation",
        classification: Box<str> => "classification",
        phenotypes: BTreeMap<PgxGeneKey, Box<str>> => "phenotypes",
        activityscore: BTreeMap<PgxGeneKey, Box<str>> => "activityscore",
        population: Box<str> => "population",
        comments: Box<str> => "comments",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
    }
}}
pgx_row! { PgxDrugRecommendation => biodata::PgxDrugRecommendation, record id {
    identity {
        id: i64 => "id" as "CPIC",
        drugid: Curie => "drugid",
        lookupkey: BTreeMap<PgxGeneKey, Box<str>> => "lookupkey",
    }
    values {
        guidelineid: i64 => "guidelineid" as "CPIC",
        implications: BTreeMap<PgxGeneKey, Box<str>> => "implications",
        drugrecommendation: Box<str> => "drugrecommendation",
        classification: Box<str> => "classification",
        phenotypes: BTreeMap<PgxGeneKey, Box<str>> => "phenotypes",
        activityscore: BTreeMap<PgxGeneKey, Box<str>> => "activityscore",
        allelestatus: BTreeMap<PgxGeneKey, Box<str>> => "allelestatus",
        population: Box<str> => "population",
        comments: Box<str> => "comments",
        version: i64 => "version",
        dosinginformation: bool => "dosinginformation",
        alternatedrugavailable: bool => "alternatedrugavailable",
        otherprescribingguidance: bool => "otherprescribingguidance",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
    }
}}
pgx_row! { PgxGuidelineGene => biodata::PgxGuidelineGene, record symbol {
    identity {
        symbol: GeneSymbol => "symbol",
    }
    values {
        url: Box<str> => "url",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
    }
}}
pgx_row! { PgxGuidelineReference => biodata::PgxGuidelineReference, record guideline_name {
    identity {
        guideline_name: Box<str> => "guideline_name",
    }
    values {
        guideline_url: Box<str> => "guideline_url",
    }
    nested {
    }
    lists {
        drugs: Box<str> => "drugs",
    }
    nested_lists {
        genes: PgxGuidelineGene => "genes",
    }
    optional_lists {
    }
}}
pgx_row! { PgxDiplotypeResult => biodata::PgxDiplotypeResult, record diplotype {
    identity {
        genesymbol: GeneSymbol => "genesymbol",
        diplotype: Box<str> => "diplotype",
    }
    values {
        function1: Box<str> => "function1",
        function2: Box<str> => "function2",
        activityvalue1: Box<str> => "activityvalue1",
        activityvalue2: Box<str> => "activityvalue2",
        totalactivityscore: Box<str> => "totalactivityscore",
        description: Box<str> => "description",
        diplotypekey: BTreeMap<PgxGeneKey, BTreeMap<Box<str>, u32>> => "diplotypekey",
        generesult: Box<str> => "generesult",
        ehrpriority: Box<str> => "ehrpriority",
        consultationtext: Box<str> => "consultationtext",
        lookupkey: BTreeMap<PgxGeneKey, Box<str>> => "lookupkey",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
    }
}}
