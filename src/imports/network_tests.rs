use super::network::{self, Network, Reply};
use reqwest::Url;
use std::{collections::VecDeque, net::SocketAddr, sync::Mutex};
struct Fake {
    dns: Mutex<VecDeque<Vec<SocketAddr>>>,
    replies: Mutex<VecDeque<Reply>>,
    calls: Mutex<Vec<(String, Vec<SocketAddr>)>>,
}
impl Fake {
    fn new(replies: Vec<Reply>) -> Self {
        Self {
            dns: Mutex::new(VecDeque::new()),
            replies: Mutex::new(replies.into()),
            calls: Mutex::new(Vec::new()),
        }
    }
}
impl Network for Fake {
    fn resolve(
        &self,
        _: &str,
    ) -> impl std::future::Future<Output = Result<Vec<SocketAddr>, &'static str>> + Send {
        std::future::ready(Ok(self
            .dns
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| vec!["93.184.215.14:443".parse().unwrap()])))
    }
    fn get(
        &self,
        url: &Url,
        addresses: &[SocketAddr],
        _: usize,
    ) -> impl std::future::Future<Output = Result<Reply, &'static str>> + Send {
        self.calls
            .lock()
            .unwrap()
            .push((url.to_string(), addresses.to_vec()));
        std::future::ready(
            self.replies
                .lock()
                .unwrap()
                .pop_front()
                .ok_or("unexpected_request"),
        )
    }
}

fn robots(text: &str) -> Reply {
    Reply {
        status: 200,
        content_type: "text/plain; charset=utf-8".to_owned(),
        body: text.as_bytes().to_vec(),
        ..Reply::default()
    }
}
fn page() -> Reply {
    Reply {
        status: 200,
        content_type: "text/html; charset=utf-8".to_owned(),
        body: b"synthetic".to_vec(),
        ..Reply::default()
    }
}
#[test]
fn unsafe_address_families_and_url_encodings_are_rejected() {
    for ip in [
        "0.1.2.3",
        "10.1.2.3",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.169.254",
        "172.16.1.2",
        "192.168.0.1",
        "192.0.2.2",
        "198.18.0.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "255.255.255.255",
        "::1",
        "::ffff:127.0.0.1",
        "64:ff9b::7f00:1",
        "fc00::1",
        "fe80::1",
        "ff00::1",
        "2001:db8::1",
        "2002:7f00:1::",
        "3fff::1",
        "100::1",
    ] {
        assert!(!network::public(ip.parse().unwrap()), "{ip}");
    }
    for ip in [
        "93.184.215.14",
        "8.8.8.8",
        "2606:4700::1111",
        "2a00:1450::1",
    ] {
        assert!(network::public(ip.parse().unwrap()));
    }
    for url in [
        "http://example.org/r",
        "https://127.0.0.1/r",
        "https://2130706433/r",
        "https://0x7f000001/r",
        "https://[::1]/r",
        "https://user:pass@example.org/r",
        "https://example.org:8080/r",
        "file:///tmp/x",
        "https://a.local/r",
    ] {
        assert!(network::url(url).is_err(), "{url}");
    }
}
#[tokio::test]
async fn mixed_dns_and_rebinding_fail_before_contacting_a_private_peer() {
    let fake = Fake::new(vec![]);
    fake.dns.lock().unwrap().push_back(vec![
        "93.184.215.14:443".parse().unwrap(),
        "10.0.0.1:443".parse().unwrap(),
    ]);
    assert_eq!(
        network::fetch(&fake, "https://example.org/r")
            .await
            .unwrap_err(),
        "import_target_blocked"
    );
    assert!(fake.calls.lock().unwrap().is_empty());
    let fake = Fake::new(vec![robots("User-agent: *\nAllow: /")]);
    fake.dns.lock().unwrap().extend([
        vec!["93.184.215.14:443".parse().unwrap()],
        vec!["169.254.169.254:443".parse().unwrap()],
    ]);
    assert_eq!(
        network::fetch(&fake, "https://example.org/r")
            .await
            .unwrap_err(),
        "import_target_blocked"
    );
    assert_eq!(fake.calls.lock().unwrap().len(), 1);
}
#[tokio::test]
async fn redirects_policy_body_limits_and_source_restrictions_are_enforced() {
    let fake = Fake::new(vec![robots(
        "User-agent: MealshopRecipeImport\nDisallow: /private",
    )]);
    assert_eq!(
        network::fetch(&fake, "https://example.org/private")
            .await
            .unwrap_err(),
        "import_robots_blocked"
    );
    assert_eq!(fake.calls.lock().unwrap().len(), 1);
    let fake = Fake::new(vec![
        robots(""),
        Reply {
            status: 302,
            location: Some("https://127.0.0.1/private".to_owned()),
            ..Reply::default()
        },
    ]);
    assert_eq!(
        network::fetch(&fake, "https://example.org/r")
            .await
            .unwrap_err(),
        "import_redirect_blocked"
    );
    assert_eq!(fake.calls.lock().unwrap().len(), 2);
    let fake = Fake::new(vec![
        robots(""),
        Reply {
            status: 302,
            location: Some("/canonical".to_owned()),
            ..Reply::default()
        },
        robots(""),
        page(),
    ]);
    assert_eq!(
        network::fetch(&fake, "https://example.org/r")
            .await
            .unwrap(),
        (
            "https://example.org/canonical".to_owned(),
            "synthetic".to_owned()
        )
    );
    for bad in [
        Reply {
            status: 403,
            ..Reply::default()
        },
        Reply {
            body: vec![b'x'; network::PAGE_LIMIT + 1],
            ..page()
        },
        Reply {
            content_type: "image/png".to_owned(),
            ..page()
        },
    ] {
        let fake = Fake::new(vec![robots(""), bad]);
        assert!(
            network::fetch(&fake, "https://example.org/r")
                .await
                .is_err()
        );
    }
}
