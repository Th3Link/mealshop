//! No authenticated client or unchecked DNS result crosses the public import boundary.
use reqwest::{Url, header};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
pub(super) const AGENT: &str = "MealshopRecipeImport";
pub(super) const PAGE_LIMIT: usize = 2_097_152;
const ROBOTS_LIMIT: usize = 524_288;
#[derive(Default)]
pub(super) struct Reply {
    pub(super) status: u16,
    pub(super) location: Option<String>,
    pub(super) content_type: String,
    pub(super) body: Vec<u8>,
}
pub(super) trait Network: Sync {
    fn resolve(
        &self,
        host: &str,
    ) -> impl std::future::Future<Output = Result<Vec<SocketAddr>, &'static str>> + Send;
    fn get(
        &self,
        url: &Url,
        addresses: &[SocketAddr],
        limit: usize,
    ) -> impl std::future::Future<Output = Result<Reply, &'static str>> + Send;
}
pub(super) struct PublicNetwork;
pub(super) fn url(raw: &str) -> Result<Url, &'static str> {
    let parsed = Url::parse(raw).map_err(|_| "import_url_invalid")?;
    let host = parsed.host_str().ok_or("import_url_invalid")?;
    if raw.len() > 2048
        || raw.chars().any(char::is_whitespace)
        || raw.chars().any(char::is_control)
        || parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port_or_known_default() != Some(443)
        || host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok()
        || !host.contains('.')
        || host.ends_with('.')
        || [
            ".localhost",
            ".local",
            ".internal",
            ".test",
            ".invalid",
            ".home.arpa",
        ]
        .iter()
        .any(|suffix| host.ends_with(suffix))
    {
        return Err("import_url_invalid");
    }
    Ok(parsed)
}
pub(super) fn public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192
                    && (b == 168 || (b == 0 && (c == 0 || c == 2)) || (b == 88 && c == 99)))
                || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(ip) => {
            let s = ip.segments();
            s[0] & 0xe000 == 0x2000
                && s[0] != 0x2002
                && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8))
                && !(s[0] == 0x3fff && s[1] & 0xf000 == 0)
        }
    }
}
async fn addresses(network: &impl Network, target: &Url) -> Result<Vec<SocketAddr>, &'static str> {
    let addresses = network
        .resolve(target.host_str().ok_or("import_url_invalid")?)
        .await?;
    if addresses.is_empty()
        || addresses.len() > 32
        || addresses.iter().any(|a| !public(a.ip()) || a.port() != 443)
    {
        return Err("import_target_blocked");
    }
    Ok(addresses)
}
fn utf8(body: Vec<u8>) -> Result<String, &'static str> {
    String::from_utf8(body).map_err(|_| "import_encoding_unsupported")
}
async fn robots(network: &impl Network, target: &Url) -> Result<(), &'static str> {
    let mut robots = target.clone();
    robots.set_path("/robots.txt");
    robots.set_query(None);
    robots.set_fragment(None);
    let reply = network
        .get(&robots, &addresses(network, &robots).await?, ROBOTS_LIMIT)
        .await?;
    match reply.status {
        400 | 404 | 405 | 406 | 410 | 414 => Ok(()),
        200 => {
            if reply
                .content_type
                .split(';')
                .next()
                .is_none_or(|mime| mime.trim() != "text/plain")
            {
                return Err("import_policy_unavailable");
            }
            if reply.body.len() > ROBOTS_LIMIT {
                return Err("import_response_large");
            }
            let text = utf8(reply.body)?;
            let permit = super::PROCESSING
                .try_acquire()
                .map_err(|_| "import_fetch_busy")?;
            let target = target.to_string();
            let allowed = tokio::task::spawn_blocking(move || {
                let _permit = permit;
                robotstxt::DefaultMatcher::default()
                    .one_agent_allowed_by_robots(&text, AGENT, &target)
            })
            .await
            .map_err(|_| "import_parser_failed")?;
            if allowed {
                Ok(())
            } else {
                Err("import_robots_blocked")
            }
        }
        _ => Err("import_policy_unavailable"),
    }
}
pub(super) async fn fetch(
    network: &impl Network,
    raw: &str,
) -> Result<(String, String), &'static str> {
    tokio::time::timeout(Duration::from_secs(15), fetch_inner(network, raw))
        .await
        .map_err(|_| "import_timeout")?
}
async fn fetch_inner(network: &impl Network, raw: &str) -> Result<(String, String), &'static str> {
    let mut target = url(raw)?;
    for redirect in 0..=3 {
        robots(network, &target).await?;
        let reply = network
            .get(&target, &addresses(network, &target).await?, PAGE_LIMIT)
            .await?;
        match reply.status {
            301 | 302 | 303 | 307 | 308 => {
                if redirect == 3 {
                    return Err("import_redirect_limit");
                }
                let next = target
                    .join(reply.location.as_deref().ok_or("import_redirect_invalid")?)
                    .map_err(|_| "import_redirect_invalid")?;
                target = url(next.as_str()).map_err(|_| "import_redirect_blocked")?;
            }
            200 => {
                let mime = reply.content_type.split(';').next().unwrap_or("").trim();
                if !matches!(
                    mime,
                    "text/html" | "application/xhtml+xml" | "application/ld+json" | "text/plain"
                ) {
                    return Err("import_type_unsupported");
                }
                if reply.content_type.split(';').skip(1).any(|parameter| {
                    parameter
                        .trim()
                        .strip_prefix("charset=")
                        .is_some_and(|charset| {
                            !matches!(charset.trim_matches('"'), "utf-8" | "utf8" | "us-ascii")
                        })
                }) {
                    return Err("import_encoding_unsupported");
                }
                if reply.body.len() > PAGE_LIMIT {
                    return Err("import_response_large");
                }
                return Ok((target.to_string(), utf8(reply.body)?));
            }
            401 | 403 | 407 | 429 | 451 => return Err("import_source_blocked"),
            _ => return Err("import_source_unavailable"),
        }
    }
    Err("import_redirect_limit")
}
impl Network for PublicNetwork {
    async fn resolve(&self, host: &str) -> Result<Vec<SocketAddr>, &'static str> {
        let records = tokio::net::lookup_host((host, 443))
            .await
            .map_err(|_| "import_dns_failed")?;
        Ok(records.take(33).collect())
    }
    async fn get(
        &self,
        url: &Url,
        addresses: &[SocketAddr],
        limit: usize,
    ) -> Result<Reply, &'static str> {
        let host = url.host_str().ok_or("import_url_invalid")?;
        let client = reqwest::Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .referer(false)
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(8))
            .resolve_to_addrs(host, addresses)
            .user_agent(format!(
                "{AGENT}/0.1 (+https://github.com/Th3Link/mealshop)"
            ))
            .build()
            .map_err(|_| "import_transport_failed")?;
        let mut response = client
            .get(url.clone())
            .header(
                header::ACCEPT,
                "text/html, application/ld+json;q=0.9, text/plain;q=0.5",
            )
            .header(header::ACCEPT_ENCODING, "identity")
            .send()
            .await
            .map_err(|_| "import_transport_failed")?;
        if !response
            .remote_addr()
            .is_some_and(|peer| addresses.contains(&peer))
        {
            return Err("import_target_blocked");
        }
        let headers = response.headers();
        if headers
            .get(header::CONTENT_ENCODING)
            .is_some_and(|v| v.as_bytes() != b"identity")
        {
            return Err("import_encoding_unsupported");
        }
        let mut reply = Reply {
            status: response.status().as_u16(),
            location: headers
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned),
            content_type: headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_ascii_lowercase(),
            body: Vec::new(),
        };
        if reply.status == 200 {
            if response.content_length().is_some_and(|n| n > limit as u64) {
                return Err("import_response_large");
            }
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "import_transport_failed")?
            {
                if reply.body.len().saturating_add(chunk.len()) > limit {
                    return Err("import_response_large");
                }
                reply.body.extend_from_slice(&chunk);
            }
        }
        Ok(reply)
    }
}
