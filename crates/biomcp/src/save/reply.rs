//! Install source bytes through typed save visitor methods.
use super::SaveTarget;
use biodata::Text;
use biomcp_types::{Reply, ReplyBody, ReplyData, Section, UsageError};

#[derive(Default)]
pub(super) struct SaveReply {
    pub(super) target: Option<SaveTarget>,
    pub(super) article: Option<(Text, Vec<u8>, Option<u32>)>,
    pub(super) pdf: Option<(biomcp_types::PdfLink, Vec<u8>)>,
    pub(super) trial: Option<(biodata::TrialDocument, Vec<u8>)>,
}
impl Reply for SaveReply {
    type Output = ReplyData;
    fn reply<B: ReplyBody>(self, query: B::Query, body: Option<B>) -> ReplyData {
        B::into_reply(query, body)
    }
    fn usage(self, error: UsageError) -> ReplyData {
        ReplyData::Usage(error)
    }
    fn pool(self, mismatch: bool) -> ReplyData {
        ReplyData::PoolError { mismatch }
    }
    fn article_text_result(
        self,
        query: biomcp_types::ArticleText,
        mut body: Option<biomcp_types::ArticleTextResult>,
    ) -> ReplyData {
        if let Some(body) = &mut body {
            match self.article_receipt(&body.content) {
                Ok(saved) => {
                    if let Section::Data { data, .. } = &mut body.content {
                        data.saved = saved;
                    }
                }
                Err(error) => return ReplyData::Usage(error),
            }
        }
        biomcp_types::ArticleTextResult::into_reply(query, body)
    }
    fn article_supplement_result(
        self,
        query: biomcp_types::ArticleSupplement,
        mut body: Option<biomcp_types::ArticleSupplementResult>,
    ) -> ReplyData {
        if let Some(body) = &mut body {
            let url = match &body.content {
                Section::Data { data, .. } => data.url.clone(),
                _ => None,
            };
            match self.receipt(url) {
                Ok(saved) => {
                    if let Section::Data { data, .. } = &mut body.content {
                        data.saved = saved;
                    }
                }
                Err(error) => return ReplyData::Usage(error),
            }
        }
        biomcp_types::ArticleSupplementResult::into_reply(query, body)
    }
    fn article_pdf_result(
        self,
        query: biomcp_types::ArticlePdf,
        mut body: Option<biomcp_types::ArticlePdfResult>,
    ) -> ReplyData {
        if let Some((link, bytes)) = self.pdf {
            let receipt = self
                .target
                .and_then(|target| target.write(&bytes).ok())
                .and_then(saved);
            let Some((path, bytes, sha256)) = receipt else {
                return ReplyData::Usage(UsageError::PdfSaveRefused(link));
            };
            body = Some(biomcp_types::ArticlePdfResult {
                file: Section::Data {
                    source: biomcp_types::Source::SemanticScholar,
                    data: biomcp_types::PdfSaved {
                        path,
                        bytes,
                        sha256,
                        pdf: link,
                    },
                },
            });
        }
        biomcp_types::ArticlePdfResult::into_reply(query, body)
    }
    fn trial_document_result(
        self,
        query: biomcp_types::TrialDocumentGet,
        mut body: Option<biomcp_types::TrialDocumentResult>,
    ) -> ReplyData {
        if let Some((document, bytes)) = self.trial {
            let receipt = self
                .target
                .and_then(|target| target.write(&bytes).ok())
                .and_then(saved);
            let Some((path, bytes, sha256)) = receipt else {
                return ReplyData::Usage(UsageError::ArticleSaveRefused(document.url));
            };
            body = Some(biomcp_types::TrialDocumentResult {
                file: Section::Data {
                    source: biomcp_types::Source::ClinicalTrialsGov,
                    data: biomcp_types::TrialDocumentSaved {
                        path,
                        bytes,
                        sha256,
                        document,
                    },
                },
            });
        }
        biomcp_types::TrialDocumentResult::into_reply(query, body)
    }
}
impl SaveReply {
    fn article_receipt(
        self,
        content: &Section<biomcp_types::TextContent>,
    ) -> Result<Option<biomcp_types::ArticleSaved>, UsageError> {
        let url = match content {
            Section::Data { data, .. } => data.url.clone(),
            _ => None,
        };
        self.receipt(url)
    }
    fn receipt(self, url: Option<Text>) -> Result<Option<biomcp_types::ArticleSaved>, UsageError> {
        if let Some((url, bytes, parts)) = self.article {
            let receipt = self
                .target
                .and_then(|target| target.write(&bytes).ok())
                .and_then(saved);
            let Some((path, bytes, sha256)) = receipt else {
                return Err(UsageError::ArticleSaveRefused(url));
            };
            Ok(Some(biomcp_types::ArticleSaved {
                path,
                bytes,
                sha256,
                url,
                parts,
            }))
        } else if let Some(url) = url {
            Err(UsageError::ArticleSaveRefused(url))
        } else {
            Ok(None)
        }
    }
}
fn saved(file: super::SavedFile) -> Option<(Text, u64, Text)> {
    Some((
        Text::new(file.path.to_str()?).ok()?,
        file.bytes,
        Text::new(&file.sha256).ok()?,
    ))
}
