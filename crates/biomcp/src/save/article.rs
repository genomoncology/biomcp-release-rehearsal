//! Article payloads reuse the single binary-owned destination and atomic write boundary.
use super::SaveTarget;
use biodata::Text;
use biomcp_query::{Ctx, Outcome, article};

pub(crate) fn text(cx: &Ctx<'_>, query: &biomcp_types::ArticleText, allow: bool) -> Outcome {
    let filename = format!("{}.md", query.input());
    let target = target(query.save(), &filename, query.replace(), allow);
    let download = article::text::download(cx, query, target.is_some());
    finish(download, target)
}
pub(crate) fn supplement(
    cx: &Ctx<'_>,
    query: &biomcp_types::ArticleSupplement,
    allow: bool,
) -> Outcome {
    let target = target(query.save(), query.file().as_str(), query.replace(), allow);
    let download = article::supplement::download(cx, query, target.is_some());
    finish(download, target)
}
fn target(path: Option<&Text>, filename: &str, replace: bool, allow: bool) -> Option<SaveTarget> {
    allow
        .then(|| SaveTarget::new(path?.as_str(), filename, replace).ok())
        .flatten()
}
fn finish(download: article::text::Download, target: Option<SaveTarget>) -> Outcome {
    let mut out = download.out;
    out.body = out.body.visit(super::reply::SaveReply {
        target,
        article: download.file.map(|file| (file.url, file.bytes, file.parts)),
        ..Default::default()
    });
    out
}
