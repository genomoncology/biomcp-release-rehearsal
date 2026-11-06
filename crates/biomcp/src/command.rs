//! Shared command interpretation and typed outcomes for CLI and MCP.
mod context;
mod encoding;
mod reply;
pub(crate) use context::Context;
pub(crate) use encoding::{Encoding, Outcome};
mod hosted_keys;
pub(crate) mod hosted_use;
pub(crate) use hosted_use::DeclaredUse;
pub(crate) mod local;
mod maintenance;
pub(crate) mod mcp_policy;
mod steps;
mod sync;
mod transport;

use biomcp_query::{Action, Bundle};
use biomcp_types::{Envelope, ErrorEnvelope, Request, Status, UsageArguments, UsageError};
use std::{ffi::OsString, io};

pub(crate) fn dispatch(
    root: &crate::grammar::Root,
    settings: &crate::config::Settings,
    args: &[OsString],
) -> io::Result<Outcome> {
    dispatch_using(
        root,
        settings,
        args,
        true,
        None,
        Encoding::cli(root, settings, args),
    )
}
fn dispatch_using(
    root: &crate::grammar::Root,
    settings: &crate::config::Settings,
    args: &[OsString],
    allow_save: bool,
    context: Option<Context<'_>>,
    encoding: Encoding,
) -> io::Result<Outcome> {
    if let [command] = args
        && command == "schema"
    {
        return encoding.encode(
            Envelope::new(
                Status::Complete,
                biomcp_types::EmptyQuery {},
                Some(crate::schema::catalog()),
            ),
            crate::exit::Status::Success,
        );
    }
    if let [command, name] = args
        && command == "schema"
        && let Some(text) = name.to_str().and_then(crate::schema::find)
    {
        return encoding.schema(text);
    }
    let mut parsed = match parse_arguments(root, args, encoding)? {
        Ok(parsed) => parsed,
        Err(outcome) => return Ok(outcome),
    };
    if let Some(context) = context
        && parsed.globals.inherit(context.globals).is_err()
    {
        return usage(root, args, encoding);
    }
    if context.is_some()
        && let Some(error) = mcp_policy::refusal(&parsed, !allow_save)
    {
        return usage_envelope(error, encoding);
    }
    if let Some(command) = parsed.key {
        return crate::keys::run(command, &parsed.globals, settings, encoding);
    }
    let Some(action) = parsed.action else {
        return usage(root, args, encoding);
    };
    if let Some(path) = parsed.help {
        return encoding.encode(
            Envelope::new(
                Status::Complete,
                biomcp_types::HelpQuery {
                    command: path.clone(),
                },
                Some(crate::help::dispatch(root, &path)?),
            ),
            crate::exit::Status::Success,
        );
    }
    if let Action::KeyHelp(source) = action {
        return key_help(source, encoding);
    }
    let action = match action.prepare() {
        Ok(action) => action,
        Err(error) => return usage_envelope(error, encoding),
    };
    let transport = match transport::fetch(&parsed.globals, settings, context)? {
        Ok(fetch) => fetch,
        Err(error) => return local::startup(error, encoding),
    };
    let settings = transport.settings.as_deref().unwrap_or(settings);
    let outcome = execute(
        root,
        settings,
        &transport.fetch,
        action,
        &parsed.globals,
        allow_save,
        encoding,
    )?;
    hosted_keys::outcome(outcome, parsed.globals.hosted, encoding)
}

fn parse_arguments(
    root: &crate::grammar::Root,
    args: &[OsString],
    encoding: Encoding,
) -> io::Result<Result<crate::grammar::Parsed, Outcome>> {
    let canonical = crate::grammar::canonical_command(root, args);
    let retired = crate::retired::message(root, canonical.as_deref().unwrap_or(args));
    let parsed = match crate::grammar::parse(root, args) {
        Ok(parsed) => parsed,
        Err(cause) => {
            let outcome = if !cause.prefer_argument_error
                && let Some(message) = retired
            {
                usage_envelope(UsageError::Retired(message), encoding)
            } else {
                parse_refusal(root, args, cause, encoding)
            };
            return Ok(Err(outcome?));
        }
    };
    match retired {
        Some(message) => Ok(Err(usage_envelope(UsageError::Retired(message), encoding)?)),
        None => Ok(Ok(parsed)),
    }
}

pub(crate) fn execute(
    root: &crate::grammar::Root,
    settings: &crate::config::Settings,
    fetch: &biomcp_fetch::Fetch,
    action: Action,
    globals: &crate::grammar::Globals,
    allow_save: bool,
    encoding: Encoding,
) -> io::Result<Outcome> {
    match action {
        Action::PubmedSearch(q) => execute(
            root,
            settings,
            fetch,
            Action::Run {
                request: Request::SearchArticle(q),
                fresh: false,
            },
            globals,
            allow_save,
            encoding,
        ),
        Action::AuthorPapers(q) => execute(
            root,
            settings,
            fetch,
            Action::Run {
                request: Request::AuthorPapers(q),
                fresh: false,
            },
            globals,
            allow_save,
            encoding,
        ),
        Action::QueryHelp(request) => {
            let path = crate::spell::command_words(&request);
            encoding.encode(
                Envelope::new(
                    Status::Complete,
                    biomcp_types::HelpQuery {
                        command: path.clone(),
                    },
                    Some(crate::help::dispatch(root, &path)?),
                ),
                crate::exit::Status::Success,
            )
        }
        Action::NetworkHelp(setting) => {
            let path = vec!["network".into(), setting.to_string()];
            encoding.encode(
                Envelope::new(
                    Status::Complete,
                    biomcp_types::HelpQuery {
                        command: path.clone(),
                    },
                    Some(crate::help::dispatch(root, &path)?),
                ),
                crate::exit::Status::Success,
            )
        }
        Action::StoreClean => maintenance::clean(fetch, encoding),
        Action::Health => maintenance::health(fetch, encoding),
        Action::KeyHelp(source) => key_help(source, encoding),
        Action::Sync { bundle } => sync::run(fetch, bundle, encoding),
        Action::Run { request, fresh } => {
            let result = run_request(fetch, &request, fresh, settings, allow_save)?;
            response(fetch, request, result, fresh, globals, encoding)
        }
    }
}
struct ResponseMeta {
    encoding: Encoding,
    status: Status,
    body_visible: bool,
    local_error: Option<biomcp_types::CacheProblem>,
    sources: Vec<biomcp_types::SourceOutcome>,
    problems: Vec<biomcp_types::Problem>,
    next: Vec<biomcp_types::Step>,
    page: Option<biomcp_types::PageInfo>,
    exit: crate::exit::Status,
}
impl ResponseMeta {
    fn encode<B: biomcp_types::ResponseBody>(
        self,
        query: B::Query,
        body: Option<B>,
    ) -> io::Result<Outcome> {
        let mut envelope = Envelope::new(self.status, query, body.filter(|_| self.body_visible));
        envelope.sources = self
            .sources
            .into_iter()
            .map(biomcp_query::sources::restore_outcome)
            .collect();
        envelope.problems = self.problems;
        envelope.next = self.next;
        envelope.page = self.page;
        envelope.local_error = self.local_error;
        self.encoding.encode(envelope, self.exit)
    }
}
fn response(
    fetch: &biomcp_fetch::Fetch,
    request: Request,
    mut result: biomcp_query::Outcome,
    fresh: bool,
    globals: &crate::grammar::Globals,
    encoding: Encoding,
) -> io::Result<Outcome> {
    result
        .finish(fetch, &request, fresh)
        .map_err(io::Error::other)?;
    let steps = steps::next(&result, globals)?;
    let meta = ResponseMeta {
        encoding,
        exit: response_status(&result),
        body_visible: result.body_visible(),
        local_error: local::kind(result.local),
        status: result.status(),
        next: steps,
        page: result.page,
        sources: result.sources,
        problems: result.problems,
    };
    result.body.visit(meta)
}
pub(crate) fn tool(args: &[OsString], context: Context<'_>) -> io::Result<Outcome> {
    tool_using(args, context, true)
}
pub(crate) fn tool_http(args: &[OsString], context: Context<'_>) -> io::Result<Outcome> {
    tool_using(args, context, false)
}
fn tool_using(args: &[OsString], context: Context<'_>, allow_save: bool) -> io::Result<Outcome> {
    if args.iter().filter_map(|arg| arg.to_str()).any(|arg| {
        ["--record", "--replay", "--case-cache"]
            .iter()
            .any(|flag| arg == *flag || arg.starts_with(&format!("{flag}=")))
    }) {
        return usage_envelope(UsageError::arguments(), Encoding::Mcp);
    }
    dispatch_using(
        context.root,
        context.settings,
        args,
        allow_save,
        Some(context),
        Encoding::Mcp,
    )
}
fn usage(
    root: &crate::grammar::Root,
    args: &[OsString],
    encoding: Encoding,
) -> io::Result<Outcome> {
    parse_refusal(
        root,
        args,
        crate::grammar::ParseInvalid::default(),
        encoding,
    )
}
fn parse_refusal(
    root: &crate::grammar::Root,
    args: &[OsString],
    cause: crate::grammar::ParseInvalid,
    encoding: Encoding,
) -> io::Result<Outcome> {
    if let Some(UsageError::Arguments(arguments)) = &cause.usage
        && arguments.argument.is_some()
        && !arguments.next.is_empty()
    {
        return usage_envelope(UsageError::Arguments(arguments.clone()), encoding);
    }
    if let Some(path) = crate::grammar::unknown_command(root, args) {
        return usage_envelope(
            UsageError::UnknownCommand {
                next: biomcp_types::Step {
                    kind: biomcp_types::StepKind::Configure,
                    args: std::iter::once("help".into())
                        .chain(path.into_iter().map(String::into_boxed_str))
                        .collect::<Vec<_>>()
                        .into(),
                },
            },
            encoding,
        );
    }
    let mut words = crate::grammar::command_words(root, args);
    let root = words.next();
    let mut arguments = match cause.usage {
        Some(UsageError::Arguments(arguments)) => arguments,
        Some(error) => return usage_envelope(error, encoding),
        None => {
            if cause.argument.is_none()
                && !matches!(
                    root,
                    Some("sync" | "key" | "store" | "version" | "mcp-config" | "skill")
                )
                && !root.is_some_and(|root| {
                    crate::grammar::command_paths()
                        .iter()
                        .any(|(verb, _, _)| *verb == root)
                })
            {
                return usage_envelope(UsageError::Schema, encoding);
            }
            UsageArguments::default()
        }
    };
    if root == Some("get") && words.next() == Some("variant") {
        arguments.message = Some(biomcp_types::ErrorMessage::VariantIdentifier);
    }
    if root == Some("mcp-config") {
        arguments.next = vec![biomcp_types::Step {
            kind: biomcp_types::StepKind::Configure,
            args: vec!["mcp-config".into(), "--client".into(), "mcp-servers".into()].into(),
        }];
    }
    if root == Some("key") {
        arguments.next = biomcp_query::sources::keys()
            .iter()
            .map(|name| biomcp_types::Step {
                kind: biomcp_types::StepKind::Configure,
                args: vec![
                    "help".into(),
                    "key".into(),
                    biomcp_query::sources::key_name(*name).into(),
                ]
                .into(),
            })
            .collect();
    }
    if let Some(option) = cause.argument {
        arguments.argument = Some(option.into());
    }
    usage_envelope(UsageError::Arguments(arguments), encoding)
}

fn key_help(name: biomcp_fetch::SecretName, encoding: Encoding) -> io::Result<Outcome> {
    let source = biomcp_query::sources::key_source(name);
    let data = biomcp_types::KeyHelp {
        guidance: Some(
            biodata::Text::new(&format!(
                "biomcp key set {}{}",
                biomcp_query::sources::key_name(name),
                biomcp_query::sources::key_suffix(name)
            ))
            .map_err(io::Error::other)?,
        ),
        source,
        config: biodata::Text::new(&format!("keys.{}", name.config())).map_err(io::Error::other)?,
        environment: biodata::Text::new(name.environment()).map_err(io::Error::other)?,
    };
    encoding.encode(
        Envelope::new(Status::Complete, biomcp_types::EmptyQuery {}, Some(data)),
        crate::exit::Status::Success,
    )
}

fn run_request(
    fetch: &biomcp_fetch::Fetch,
    request: &Request,
    fresh: bool,
    settings: &crate::config::Settings,
    allow_save: bool,
) -> io::Result<biomcp_query::Outcome> {
    let configured_request = crate::config::request_defaults(settings, request)
        .map_err(|_| io::Error::other("local configuration failed"))?;
    let request = &configured_request;
    let cx = biomcp_query::Ctx {
        ncbi_email: crate::config::ncbi_email(settings)
            .map_err(|_| io::Error::other("local configuration failed"))?,
        fetch,
        budget: crate::config::budget(settings),
        fresh,
    };
    Ok(match request {
        Request::TrialDocument(query) => crate::save::trial::document(&cx, query, allow_save),
        Request::ArticlePdf(query) => crate::save::run(&cx, query, allow_save),
        Request::ArticleText(query) if query.save().is_some() => {
            crate::save::article::text(&cx, query, allow_save)
        }
        Request::ArticleSupplement(query) if query.save().is_some() => {
            crate::save::article::supplement(&cx, query, allow_save)
        }
        _ => biomcp_query::run(&cx, request),
    })
}

fn pool_error(
    mismatch: bool,
    steps: Vec<biomcp_types::Step>,
    encoding: Encoding,
) -> io::Result<Outcome> {
    let restart = steps
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing restart"))?;
    if mismatch {
        usage_envelope(UsageError::PoolMismatch { restart }, encoding)
    } else {
        encoding.encode(
            ErrorEnvelope::expired_pool(restart),
            crate::exit::Status::Expired,
        )
    }
}

pub(crate) fn usage_envelope(error: UsageError, encoding: Encoding) -> io::Result<Outcome> {
    encoding.encode(ErrorEnvelope::from(error), crate::exit::Status::Usage)
}

fn response_status(result: &biomcp_query::Outcome) -> crate::exit::Status {
    if result.local.is_some() {
        crate::exit::Status::Local
    } else {
        crate::exit::Status::from(result.status())
    }
}

fn bundle_source(bundle: Bundle) -> biomcp_types::Source {
    bundle.source()
}
