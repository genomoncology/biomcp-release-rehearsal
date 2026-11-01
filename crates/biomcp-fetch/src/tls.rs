//! The ureq connector boundary adds roots without replacing platform verification.
use crate::FetchError;
use rustls::{
    ClientConfig, ClientConnection, StreamOwned,
    pki_types::{CertificateDer, ServerName},
};
use std::{
    io::{Read, Write},
    sync::Arc,
};
use ureq::{
    Error,
    unversioned::transport::{
        Buffers, ConnectionDetails, Connector, Either, LazyBuffers, NextTimeout, Transport,
        TransportAdapter,
    },
};

pub(crate) fn configuration(
    roots: Vec<CertificateDer<'static>>,
) -> Result<ClientConfig, FetchError> {
    #[cfg(test)]
    crate::tls_build_tests::BUILDS.with(|count| count.set(count.get() + 1));
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier =
        rustls_platform_verifier::Verifier::new_with_extra_roots(roots, provider.clone())
            .map_err(|_| FetchError::Certificates)?;
    Ok(ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| FetchError::Certificates)?
        // The platform verifier checks the chain, name, time and handshake signatures.
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth())
}
#[derive(Debug, Default)]
pub(crate) struct ProxyConnector(ureq::unversioned::transport::ConnectProxyConnector);
impl<In: Transport> Connector<In> for ProxyConnector {
    type Out = Either<In, Box<dyn Transport>>;
    fn connect(
        &self,
        details: &ConnectionDetails<'_>,
        chained: Option<In>,
    ) -> Result<Option<Self::Out>, Error> {
        // Only this connector's failures are proxy-stage failures. A later target
        // TLS or HTTP failure retains its own class even through a working tunnel.
        self.0
            .connect(details, chained)
            .map_err(|_| Error::ConnectProxyFailed(String::new()))
    }
}
#[derive(Debug)]
pub(crate) struct TlsConnector(pub(crate) Arc<ClientConfig>);
impl<In: Transport> Connector<In> for TlsConnector {
    type Out = Either<In, TlsTransport>;
    fn connect(
        &self,
        details: &ConnectionDetails<'_>,
        chained: Option<In>,
    ) -> Result<Option<Self::Out>, Error> {
        let Some(transport) = chained else {
            return Err(Error::ConnectionFailed);
        };
        if !details.needs_tls() || transport.is_tls() {
            return Ok(Some(Either::A(transport)));
        }
        let name = details.uri.host().unwrap_or("").trim_matches(['[', ']']);
        let name =
            ServerName::try_from(name.to_owned()).map_err(|_| Error::Tls("invalid server name"))?;
        let mut conn = ClientConnection::new(self.0.clone(), name)?;
        let mut sock = TransportAdapter::new(transport.boxed(), details.current_time.clone());
        let mut timeout = details.timeout;
        use ureq::unversioned::transport::time::{Duration, Instant};
        if let (Duration::Exact(limit), Instant::Exact(now), Instant::Exact(start)) =
            (timeout.after, (details.current_time)(), details.now)
        {
            timeout.after =
                Duration::Exact(limit.saturating_sub(now.saturating_duration_since(start)));
        }
        sock.set_timeout(timeout);
        conn.complete_io(&mut sock).map_err(tls_io)?;
        Ok(Some(Either::B(TlsTransport {
            buffers: LazyBuffers::new(
                details.config.input_buffer_size(),
                details.config.output_buffer_size(),
            ),
            stream: StreamOwned { conn, sock },
        })))
    }
}
fn tls_io(error: std::io::Error) -> Error {
    if error
        .get_ref()
        .is_some_and(|inner| inner.is::<rustls::Error>())
    {
        Error::Tls("certificate or handshake verification failed")
    } else {
        error.into()
    }
}
pub(crate) struct TlsTransport {
    buffers: LazyBuffers,
    stream: StreamOwned<ClientConnection, TransportAdapter>,
}
impl Transport for TlsTransport {
    fn buffers(&mut self) -> &mut dyn Buffers {
        &mut self.buffers
    }
    fn transmit_output(&mut self, amount: usize, timeout: NextTimeout) -> Result<(), Error> {
        self.stream.get_mut().set_timeout(timeout);
        let bytes = self
            .buffers
            .output()
            .get(..amount)
            .ok_or(Error::ConnectionFailed)?;
        self.stream.write_all(bytes).map_err(tls_io)
    }
    fn await_input(&mut self, timeout: NextTimeout) -> Result<bool, Error> {
        self.stream.get_mut().set_timeout(timeout);
        let count = self
            .stream
            .read(self.buffers.input_append_buf())
            .map_err(tls_io)?;
        self.buffers.input_appended(count);
        Ok(count > 0)
    }
    fn is_open(&mut self) -> bool {
        self.stream.get_mut().get_mut().is_open()
    }
    fn is_tls(&self) -> bool {
        true
    }
}
pub(crate) fn reason(error: &Error) -> biomcp_types::Reason {
    match error {
        Error::ConnectProxyFailed(_) => biomcp_types::Reason::Proxy,
        Error::Io(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            biomcp_types::Reason::Rejected
        }
        Error::Tls(_) | Error::Rustls(_) => biomcp_types::Reason::Tls,
        Error::Timeout(_) if !crate::retry::retry_error(error) => {
            biomcp_types::Reason::DeadlineExceeded
        }
        _ => biomcp_types::Reason::Unavailable,
    }
}

impl std::fmt::Debug for TlsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsTransport").finish_non_exhaustive()
    }
}
