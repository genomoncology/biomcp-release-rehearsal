//! Checked process network settings and separate pools for each route.
use crate::{FetchError, Secret};
use rustls::pki_types::{CertificateDer, pem::PemObject};
use std::{fmt, sync::Arc};
mod bypass;
use bypass::Bypass;

/// Checked additional trust anchors. Formatting never exposes certificate bytes.
#[derive(Clone, Default)]
pub struct Certificates(Vec<CertificateDer<'static>>);
impl Certificates {
    /// Decode an explicit PEM bundle and reject empty or malformed certificate sets.
    pub fn from_pem(bytes: &[u8]) -> Result<Self, FetchError> {
        let certs: Vec<_> = CertificateDer::pem_slice_iter(bytes)
            .collect::<Result<_, _>>()
            .map_err(|_| FetchError::Certificates)?;
        if certs.is_empty() {
            return Err(FetchError::Certificates);
        }
        let mut roots = rustls::RootCertStore::empty();
        for cert in &certs {
            roots
                .add(cert.clone())
                .map_err(|_| FetchError::Certificates)?;
        }
        Ok(Self(certs))
    }
}
impl fmt::Debug for Certificates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Certificates")
            .field("count", &self.0.len())
            .finish()
    }
}
/// Owned, checked routing and trust settings; construction never reads the environment.
#[derive(Clone, Default)]
pub struct Network {
    http: Option<ureq::Proxy>,
    https: Option<ureq::Proxy>,
    bypass: Vec<Bypass>,
    roots: Certificates,
    configuration: biomcp_types::ConfigurationHealth,
}
impl fmt::Debug for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Network")
            .field("roots", &self.roots)
            .finish_non_exhaustive()
    }
}
impl Network {
    pub(crate) fn requires_proxy(&self, explicit: bool, uri: &ureq::http::Uri) -> bool {
        !matches!(self.route(explicit, uri), Route::Direct)
    }
    fn route(&self, explicit: bool, uri: &ureq::http::Uri) -> Route {
        if !explicit && self.bypass.iter().any(|rule| rule.matches(uri)) {
            return Route::Direct;
        }
        if uri.scheme_str() == Some("https") {
            if explicit || self.https.is_some() {
                Route::Https
            } else {
                Route::Direct
            }
        } else if explicit || self.http.is_some() {
            Route::Http
        } else {
            Route::Direct
        }
    }
    pub(crate) fn rpc_proxy(
        &self,
        explicit: Option<&Secret>,
        uri: &ureq::http::Uri,
    ) -> Result<Option<ureq::Proxy>, FetchError> {
        if explicit.is_some() {
            return proxy(explicit);
        }
        Ok(match self.route(false, uri) {
            Route::Direct => None,
            Route::Http => self.http.clone(),
            Route::Https => self.https.clone(),
        })
    }
    pub(crate) fn tls_configuration(&self) -> Result<rustls::ClientConfig, FetchError> {
        crate::tls::configuration(self.roots.0.clone())
    }

    /// Whether unsupported bypass entries were skipped; never returns their text.
    pub(super) fn configuration(&self) -> Option<&biomcp_types::ConfigurationHealth> {
        (!self.configuration.notices.is_empty()).then_some(&self.configuration)
    }

    /// Check HTTP/HTTPS proxy URLs, comma-separated bypass rules and additive roots.
    /// Bypass supports exact hosts/IPs, .domain (including the domain), *.domain
    /// (subdomains only), IP CIDR ranges, optional ports, and * for every destination.
    /// Unsupported entries are skipped; credentials and malformed ports refuse.
    pub fn new(
        http: Option<Secret>,
        https: Option<Secret>,
        no_proxy: &str,
        roots: Certificates,
    ) -> Result<Self, FetchError> {
        let mut bypass = Vec::new();
        let mut configuration = biomcp_types::ConfigurationHealth::default();
        for (index, entry) in no_proxy.split(',').map(str::trim).enumerate() {
            if entry.is_empty() {
                continue;
            }
            match Bypass::parse(entry)? {
                Some(rule) => bypass.push(rule),
                None if configuration.notices.len() < 8 => {
                    configuration
                        .notices
                        .push(biomcp_types::ConfigurationNotice {
                            variable: biomcp_types::ConfigurationVariable::NoProxy,
                            entry: u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1),
                            code: biomcp_types::ConfigurationNoticeCode::UnparseableEntry,
                        })
                }
                None => {
                    configuration.omitted_entries = configuration.omitted_entries.saturating_add(1)
                }
            }
        }
        Ok(Self {
            http: proxy(http.as_ref()).map_err(|_| FetchError::HttpProxy)?,
            https: proxy(https.as_ref()).map_err(|_| FetchError::HttpsProxy)?,
            roots,
            bypass,
            configuration,
        })
    }
}
fn proxy(value: Option<&Secret>) -> Result<Option<ureq::Proxy>, FetchError> {
    value
        .map(|value| {
            let proxy = ureq::Proxy::new(value.value()).map_err(|_| FetchError::Proxy)?;
            if !matches!(proxy.uri().scheme_str(), Some("http" | "https")) {
                return Err(FetchError::Proxy);
            }
            Ok(proxy)
        })
        .transpose()
}
pub(crate) struct Agents {
    direct: ureq::Agent,
    publisher: ureq::Agent,
    http: ureq::Agent,
    https: ureq::Agent,
}
impl Agents {
    pub(crate) fn shutdown(&self) {
        self.direct.clone().shutdown();
        self.publisher.clone().shutdown();
        self.http.clone().shutdown();
        self.https.clone().shutdown();
    }
    pub(crate) fn new(
        network: &Network,
        explicit: Option<&Secret>,
        tls: Arc<rustls::ClientConfig>,
    ) -> Result<Self, FetchError> {
        let explicit = proxy(explicit)?;
        let make = |proxy| {
            use ureq::unversioned::{
                resolver::DefaultResolver,
                transport::{Connector, TcpConnector},
            };
            let config = ureq::Agent::config_builder()
                .proxy(proxy)
                .http_status_as_error(false)
                .max_redirects(0)
                .build();
            let connector =
                ().chain(crate::tls::ProxyConnector::default())
                    .chain(TcpConnector::default())
                    .chain(crate::tls::TlsConnector(tls.clone()));
            ureq::Agent::with_parts(config, connector, DefaultResolver::default())
        };
        let publisher = {
            use ureq::unversioned::transport::{Connector, TcpConnector};
            let config = ureq::Agent::config_builder()
                .proxy(None)
                .http_status_as_error(false)
                .max_redirects(0)
                .build();
            let connector =
                ().chain(TcpConnector::default())
                    .chain(crate::tls::TlsConnector(tls.clone()));
            ureq::Agent::with_parts(
                config,
                connector,
                crate::destination::PublicResolver::new(network.clone(), explicit.is_some()),
            )
        };
        Ok(Self {
            direct: make(None),
            publisher,
            http: make(explicit.clone().or_else(|| network.http.clone())),
            https: make(explicit.or_else(|| network.https.clone())),
        })
    }
    pub(crate) fn publisher(&self) -> &ureq::Agent {
        &self.publisher
    }
    pub(crate) fn for_url(&self, network: &Network, explicit: bool, url: &str) -> &ureq::Agent {
        let Ok(uri) = url.parse::<ureq::http::Uri>() else {
            return &self.direct;
        };
        match network.route(explicit, &uri) {
            Route::Direct => &self.direct,
            Route::Http => &self.http,
            Route::Https => &self.https,
        }
    }
}

#[derive(Clone, Copy)]
enum Route {
    Direct,
    Http,
    Https,
}
