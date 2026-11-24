use crate::SessionError;
use reqwest::{blocking::ClientBuilder, Url};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// HTTPS origin whose local listener can differ from the advertised address.
#[derive(Clone)]
pub struct NodeEndpoint {
    url: Url,
    pub(crate) address: Option<SocketAddr>,
}

impl NodeEndpoint {
    pub fn new(origin: &str) -> Result<Self, SessionError> {
        let url = Url::parse(origin)?;
        if url.scheme() != "https"
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("expected an HTTPS coordinator origin".into());
        }
        Ok(Self { url, address: None })
    }

    pub fn bound(origin: &str, listener: SocketAddr) -> Result<Self, SessionError> {
        let mut endpoint = Self::new(origin)?;
        if endpoint.url.domain().is_some() {
            let ip = match listener.ip() {
                IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
                IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
                ip => ip,
            };
            endpoint
                .url
                .set_port(Some(listener.port()))
                .map_err(|_| "invalid local listener port")?;
            endpoint.address = Some(SocketAddr::new(ip, listener.port()));
        }
        Ok(endpoint)
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn configure(&self, builder: ClientBuilder) -> ClientBuilder {
        match (self.address, self.url.domain()) {
            (Some(address), Some(host)) => builder.resolve(host, address),
            _ => builder,
        }
    }
}
