//! Command transport selection preserves server-owned resources under offline restrictions.
use crate::{command::Context, config::StartupError, grammar::Globals};
use biomcp_fetch::{Fetch, Secrets};
use std::io;
use std::sync::Arc;
pub(super) struct Transport {
    pub(super) fetch: Fetch,
    pub(super) settings: Option<Arc<crate::config::Settings>>,
}

pub(super) fn fetch(
    globals: &Globals,
    settings: &crate::config::Settings,
    context: Option<Context<'_>>,
) -> io::Result<Result<Transport, StartupError>> {
    if let Some(context) = context
        && let Some(recovery) = context.recovery
    {
        let (ready, fresh) = match recovery.snapshot(context.globals) {
            Ok(ready) => ready,
            Err(error) => return Ok(Err(error)),
        };
        let credentials = if fresh {
            match crate::config::secrets(&ready.settings) {
                Ok(secrets) => Some(secrets),
                Err(error) => return Ok(Err(error.into())),
            }
        } else {
            None
        };
        let fetch = if globals.offline {
            ready.fetch.offline()
        } else {
            ready.fetch.as_ref().clone()
        };
        return snapshot(fetch, globals, &ready.settings, Some(context), credentials).map(
            |result| {
                result.map(|fetch| Transport {
                    fetch,
                    settings: Some(ready.settings),
                })
            },
        );
    }
    fetch_using(globals, settings, context).map(|result| {
        result.map(|fetch| Transport {
            fetch,
            settings: None,
        })
    })
}

fn fetch_using(
    globals: &Globals,
    settings: &crate::config::Settings,
    context: Option<Context<'_>>,
) -> io::Result<Result<Fetch, StartupError>> {
    match context.map(|context| context.fetch) {
        Some(Ok(fetch)) => {
            let fetch = if globals.offline {
                fetch.offline()
            } else {
                fetch.as_ref().clone()
            };
            snapshot(fetch, globals, settings, context, None)
        }
        Some(Err(error)) if !globals.offline => Ok(Err(error.clone())),
        _ if globals.hosted => match crate::config::fetch(globals, settings) {
            Ok(fetch) => snapshot(fetch, globals, settings, context, None),
            Err(error) => Ok(Err(error)),
        },
        _ => Ok(crate::config::fetch(globals, settings)),
    }
}
fn snapshot(
    fetch: Fetch,
    globals: &Globals,
    settings: &crate::config::Settings,
    context: Option<Context<'_>>,
    credentials: Option<Secrets>,
) -> io::Result<Result<Fetch, StartupError>> {
    let fetch = if let Some(scope) = context.and_then(|context| context.scope) {
        match fetch.with_cancellation(scope) {
            Ok(fetch) => fetch,
            Err(_) => return Ok(Err(StartupError::Local)),
        }
    } else {
        fetch
    };
    let credentials = match credentials {
        Some(secrets) => Ok(secrets),
        None if context.is_some() => crate::config::credentials::refresh(globals),
        None => crate::config::secrets(settings),
    };
    let secrets = match credentials {
        Ok(secrets) => secrets,
        Err(error) => return Ok(Err(error.into())),
    };
    let fetch = if globals.hosted {
        fetch.with_hosted_secrets(
            secrets.hosted(
                context.and_then(|context| context.secrets),
                biomcp_query::sources::keys()
                    .iter()
                    .copied()
                    .filter(|&name| {
                        biomcp_query::sources::key_tier(name) == biomcp_types::KeyTier::NoFallback
                    }),
            ),
        )
    } else {
        fetch.with_secrets(secrets)
    };
    let fetch = crate::config::restrict(fetch, globals)?;
    Ok(Ok(super::hosted_use::restrict(fetch, context)))
}
