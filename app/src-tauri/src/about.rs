//! Settings → About and the Help menu: the version and the project's links.
//! Every link is a constant or is built here from the app's own version, OS
//! and architecture. Nothing from a prompt or a library goes into a URL.

use serde::Serialize;

pub const REPO: &str = "https://github.com/ETSoftwareStudio/promptshelf";
pub const SITE: &str = "https://etsoftware.studio";

/// The version release-please writes into Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub repo: String,
    pub releases: String,
    pub issue: String,
    pub licence: String,
    pub site: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutView {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub links: Links,
}

pub fn view() -> AboutView {
    AboutView {
        version: VERSION.into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        links: Links {
            repo: REPO.into(),
            releases: format!("{REPO}/releases"),
            issue: issue_url(),
            licence: format!("{REPO}/blob/main/LICENSE"),
            site: SITE.into(),
        },
    }
}

/// A new issue with the version, the OS and the architecture in its body.
/// Nothing else about the computer goes in.
pub fn issue_url() -> String {
    issue_url_for(VERSION, std::env::consts::OS, std::env::consts::ARCH)
}

fn issue_url_for(version: &str, os: &str, arch: &str) -> String {
    let body = format!(
        "**What happened**\n\n\n**What you expected**\n\n\n---\nPromptShelf {version}, {os}, {arch}\n"
    );
    format!("{REPO}/issues/new?body={}", encode(&body))
}

/// Percent-encodes a query value: every byte except the unreserved ones.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Opens a web page in the system browser.
pub fn open(url: &str) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(format!("{url} is not a web address"));
    }
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("could not open {url}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_issue_body_has_the_version_os_and_arch_only() {
        let url = issue_url_for("1.2.3", "macos", "aarch64");
        assert_eq!(
            url,
            format!(
                "{REPO}/issues/new?body=%2A%2AWhat%20happened%2A%2A%0A%0A%0A%2A%2AWhat%20you%20expected%2A%2A%0A%0A%0A---%0APromptShelf%201.2.3%2C%20macos%2C%20aarch64%0A"
            )
        );
    }

    #[test]
    fn encode_keeps_unreserved_and_escapes_the_rest() {
        assert_eq!(encode("a-Z_0.~"), "a-Z_0.~");
        assert_eq!(encode("a b&c=d/é"), "a%20b%26c%3Dd%2F%C3%A9");
    }

    #[test]
    fn every_link_is_on_github_or_the_studio_site() {
        let v = view();
        for url in [
            &v.links.repo,
            &v.links.releases,
            &v.links.issue,
            &v.links.licence,
        ] {
            assert!(url.starts_with(REPO), "{url}");
        }
        assert_eq!(v.links.site, SITE);
        assert_eq!(v.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn open_refuses_other_schemes() {
        assert!(open("file:///etc/passwd").is_err());
        assert!(open("javascript:alert(1)").is_err());
    }
}
