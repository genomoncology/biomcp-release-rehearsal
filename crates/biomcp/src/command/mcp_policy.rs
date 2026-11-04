//! Transport authority is checked on parsed commands before fetch or save work.
use biomcp_query::Action;
use biomcp_types::{ErrorMessage, Request, UsageError};

pub(super) fn refusal(parsed: &crate::grammar::Parsed, http: bool) -> Option<UsageError> {
    let globals = &parsed.globals;
    let action = parsed.action.as_ref();
    if globals.hosted
        && (parsed.key.is_some()
            || globals.fresh
            || matches!(
                action,
                Some(Action::Health | Action::Sync { .. } | Action::StoreClean)
            )
            || matches!(action, Some(Action::Run { request, .. }) if saves(request)))
    {
        return Some(UsageError::McpUnavailable(ErrorMessage::McpHosted));
    }
    if parsed.key.is_some() {
        return Some(UsageError::McpUnavailable(ErrorMessage::McpKey));
    }
    let action = action?;
    if http {
        let command = match action {
            Action::Sync { .. } => Some(ErrorMessage::McpSync),
            Action::StoreClean => Some(ErrorMessage::McpStoreClean),
            _ => None,
        };
        if let Some(command) = command {
            return Some(UsageError::McpUnavailable(command));
        }
        if globals.fresh {
            return Some(UsageError::arguments());
        }
    }
    match action {
        Action::Run { request, .. } if replaces(request) || hidden_save(request) => {
            Some(UsageError::arguments())
        }
        _ => None,
    }
}
fn replaces(request: &Request) -> bool {
    match request {
        Request::ArticlePdf(query) => query.replace,
        Request::TrialDocument(query) => query.replace,
        Request::ArticleText(query) => query.replace(),
        Request::ArticleSupplement(query) => query.replace(),
        _ => false,
    }
}

fn saves(request: &Request) -> bool {
    match request {
        Request::ArticlePdf(_) | Request::TrialDocument(_) => true,
        Request::ArticleText(query) => query.save().is_some(),
        Request::ArticleSupplement(query) => query.save().is_some(),
        _ => false,
    }
}

fn hidden_save(request: &Request) -> bool {
    let destination = match request {
        Request::ArticlePdf(query) => Some(&query.save),
        Request::TrialDocument(query) => Some(&query.save),
        Request::ArticleText(query) => query.save(),
        Request::ArticleSupplement(query) => query.save(),
        _ => None,
    };
    destination.is_some_and(|path| {
        let path = std::path::Path::new(path.as_str());
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        path.as_os_str()
            .to_string_lossy()
            .split(['/', '\\'])
            .any(|part| part.starts_with('.'))
            || std::fs::canonicalize(path)
                .or_else(|_| std::fs::canonicalize(parent))
                .is_ok_and(|resolved| dotted(&resolved))
    })
}
fn dotted(path: &std::path::Path) -> bool {
    path.iter()
        .any(|part| part.to_string_lossy().starts_with('.'))
}
