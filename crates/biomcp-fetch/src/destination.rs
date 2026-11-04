//! Publisher URLs and DNS answers checked at the connection boundary.
use crate::request::Failure;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use ureq::unversioned::{
    resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver},
    transport::NextTimeout,
};

/// Validate a copied HTTP(S) annotation without applying download destination policy.
/// The caller retains the original spelling; fragments and IP hosts remain valid annotations.
pub fn check_annotation_url(raw: &str) -> Result<(), crate::InputError> {
    let parsed = url::Url::parse(raw).map_err(|_| crate::InputError::Path)?;
    if matches!(parsed.scheme(), "http" | "https") && parsed.host().is_some() {
        Ok(())
    } else {
        Err(crate::InputError::Path)
    }
}

pub(crate) fn checked(raw: &str) -> Result<String, crate::InputError> {
    let url = url::Url::parse(raw).map_err(|_| crate::InputError::Path)?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !matches!(url.host(), Some(url::Host::Domain(_)))
        || url
            .host_str()
            .is_none_or(|host| host.is_empty() || host.split('.').any(|label| label.is_empty()))
    {
        return Err(crate::InputError::Path);
    }
    Ok(url.into())
}

#[derive(Debug)]
pub(crate) struct PublicResolver {
    resolver: DefaultResolver,
    network: crate::Network,
    explicit: bool,
}
impl PublicResolver {
    pub(crate) fn new(network: crate::Network, explicit: bool) -> Self {
        Self {
            resolver: DefaultResolver::default(),
            network,
            explicit,
        }
    }
}
impl Resolver for PublicResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        checked(&uri.to_string()).map_err(|_| rejected())?;
        if self.network.requires_proxy(self.explicit, uri) {
            return Err(ureq::Error::ConnectProxyFailed(String::new()));
        }
        let addresses = self.resolver.resolve(uri, config, timeout)?;
        if addresses.iter().any(|address| !public(address.ip())) {
            return Err(rejected());
        }
        // The TCP connector receives these exact checked socket addresses.
        // It never resolves the publisher hostname again.
        Ok(addresses)
    }
}
fn public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => v4(ip),
        IpAddr::V6(ip) => v6(ip),
    }
}
fn v4(ip: Ipv4Addr) -> bool {
    let n = u32::from(ip);
    const RESERVED: &[(u32, u32)] = &[
        (0x0000_0000, 8),
        (0x0a00_0000, 8),
        (0x6440_0000, 10),
        (0x7f00_0000, 8),
        (0xa9fe_0000, 16),
        (0xac10_0000, 12),
        (0xc000_0000, 24),
        (0xc000_0200, 24),
        (0xc058_6300, 24),
        (0xc0a8_0000, 16),
        (0xc612_0000, 15),
        (0xc633_6400, 24),
        (0xcb00_7100, 24),
        (0xe000_0000, 3),
    ];
    !RESERVED
        .iter()
        .any(|(network, prefix)| n >> (32 - prefix) == network >> (32 - prefix))
}
fn v6(ip: Ipv6Addr) -> bool {
    let n = u128::from(ip);
    // Require native global unicast, excluding protocol assignments, documentation
    // and transition ranges that could tunnel to an unvalidated IPv4 destination.
    n >> 125 == 1
        && n >> 96 != 0x2001_0db8
        && n >> 105 != 0x2001_0000_u128 >> 9
        && n >> 112 != 0x2002
        && n >> 108 != 0x3fff0
}

impl crate::Leg<'_> {
    pub(crate) fn publisher_call(
        &self,
        initial: &str,
        accept: &str,
    ) -> Result<ureq::http::Response<ureq::Body>, Failure> {
        let transport = Failure::Transport;
        let mut url = checked(initial).map_err(|_| transport(rejected()))?;
        for redirects in 0..=3 {
            self.spec
                .admit(&url, true)
                .map_err(|error| Failure::Local(error.into()))?;
            let uri = url
                .parse::<ureq::http::Uri>()
                .map_err(|_| transport(rejected()))?;
            if self
                .fetch
                .config
                .network
                .requires_proxy(self.fetch.config.proxy.is_some(), &uri)
            {
                return Err(transport(ureq::Error::ConnectProxyFailed(String::new())));
            }
            let remaining = self.budget().remaining();
            if remaining.is_zero() {
                return Err(transport(ureq::Error::Timeout(ureq::Timeout::Global)));
            }
            let response = self
                .fetch
                .agents
                .publisher()
                .get(&url)
                .header("Accept", accept)
                .config()
                .timeout_global(Some(remaining))
                .build()
                .call()
                .map_err(transport)?;
            if !matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308) {
                return Ok(response);
            }
            if redirects == 3 {
                return Err(transport(rejected()));
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| transport(rejected()))?;
            url = redirect(&url, location).map_err(transport)?;
        }
        Err(transport(rejected()))
    }
}
fn rejected() -> ureq::Error {
    ureq::Error::Io(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "publisher destination rejected",
    ))
}

fn redirect(base: &str, location: &str) -> Result<String, ureq::Error> {
    let next = url::Url::parse(base)
        .map_err(|_| rejected())?
        .join(location)
        .map_err(|_| rejected())?;
    checked(next.as_str()).map_err(|_| rejected())
}
#[cfg(test)]
#[path = "destination_tests.rs"]
mod tests;
