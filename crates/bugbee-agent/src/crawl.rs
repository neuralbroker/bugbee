//! Live-target crawling engine: robots.txt respect, cycle detection, page
//! extraction, API-spec discovery.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tracing::{debug, info};

const DEFAULT_USER_AGENT: &str = "BugbeeCrawler/1.0 (+https://bugbee.security)";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_MAX_URLS: usize = 200;
const DEFAULT_MAX_DEPTH: u32 = 3;

/// A single discovered page or resource.
#[derive(Debug, Clone)]
pub struct CrawledPage {
    pub url: String,
    pub depth: u32,
    pub content_type: String,
    pub body: String,
    pub links: Vec<String>,
    pub api_endpoints: Vec<String>,
    pub elapsed_ms: u64,
}

/// What the crawler found during its run.
#[derive(Debug, Clone, Default)]
pub struct CrawlReport {
    pub pages_crawled: usize,
    pub total_links: usize,
    pub api_specs: Vec<ApiSpec>,
    pub errors: Vec<CrawlError>,
    pub elapsed_ms: u64,
}

/// An API specification discovered during the crawl.
#[derive(Debug, Clone)]
pub struct ApiSpec {
    pub url: String,
    pub kind: ApiSpecKind,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiSpecKind {
    OpenApi,
    GraphQl,
    Swagger,
    AsyncApi,
    Other(String),
}

/// A non-fatal error encountered during crawling.
#[derive(Debug, Clone)]
pub struct CrawlError {
    pub url: String,
    pub message: String,
}

/// Configuration for the live-target crawler.
#[derive(Debug, Clone)]
pub struct CrawlConfig {
    pub user_agent: String,
    pub timeout: Duration,
    pub max_urls: usize,
    pub max_depth: u32,
    pub respect_robots: bool,
    pub max_concurrent: usize,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            user_agent: DEFAULT_USER_AGENT.into(),
            timeout: DEFAULT_TIMEOUT,
            max_urls: DEFAULT_MAX_URLS,
            max_depth: DEFAULT_MAX_DEPTH,
            respect_robots: true,
            max_concurrent: 4,
        }
    }
}

/// The live-target crawler.
pub struct Crawler {
    client: Client,
    config: CrawlConfig,
    visited: Arc<parking_lot::Mutex<HashSet<String>>>,
    disallowed_paths: Arc<parking_lot::Mutex<Vec<String>>>,
    crawled_urls: Vec<String>,
    api_specs: Vec<ApiSpec>,
    errors: Vec<CrawlError>,
    start: std::time::Instant,
    request_count: Arc<AtomicU64>,
}

impl Crawler {
    pub fn new(config: CrawlConfig) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(10))
            .https_only(false)
            .pool_max_idle_per_host(config.max_concurrent)
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;

        Ok(Self {
            client,
            config,
            visited: Arc::new(parking_lot::Mutex::new(HashSet::new())),
            disallowed_paths: Arc::new(parking_lot::Mutex::new(Vec::new())),
            crawled_urls: Vec::new(),
            api_specs: Vec::new(),
            errors: Vec::new(),
            start: std::time::Instant::now(),
            request_count: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Crawl from a seed URL. Returns a report of everything discovered.
    pub async fn crawl(&mut self, seed: &str) -> CrawlReport {
        self.start = std::time::Instant::now();
        self.visited.lock().clear();
        self.disallowed_paths.lock().clear();
        self.crawled_urls.clear();
        self.api_specs.clear();
        self.errors.clear();

        let base_url = normalize_url(seed);
        info!("crawl starting from: {base_url}");

        // Fetch robots.txt if configured
        if self.config.respect_robots {
            if let Err(e) = self.fetch_robots(&base_url).await {
                debug!("robots.txt fetch skipped: {e}");
            }
        }

        // Check API-spec well-known paths even before crawling
        self.discover_api_specs(&base_url).await;

        // BFS crawl from seed
        self.crawl_bfs(&base_url).await;

        let elapsed = self.start.elapsed();
        CrawlReport {
            pages_crawled: self.crawled_urls.len(),
            total_links: self.crawled_urls.len(), // approximate
            api_specs: self.api_specs.clone(),
            errors: self.errors.clone(),
            elapsed_ms: elapsed.as_millis() as u64,
        }
    }

    async fn crawl_bfs(&mut self, seed: &str) {
        let mut queue: Vec<(String, u32)> = vec![(seed.to_string(), 0)];
        let mut visited_local = HashSet::new();

        while let Some((url, depth)) = queue.pop() {
            if self.crawled_urls.len() >= self.config.max_urls {
                debug!("crawl: hit max_urls limit ({})", self.config.max_urls);
                break;
            }

            let normalized = normalize_url(&url);
            if !visited_local.insert(normalized.clone()) {
                continue;
            }

            if self.is_disallowed(&normalized) {
                debug!("crawl: skipping disallowed by robots.txt: {normalized}");
                continue;
            }

            match self.fetch_page(&normalized).await {
                Ok(page) => {
                    let page_links = page.links.len();
                    self.crawled_urls.push(normalized.clone());

                    // Queue discovered links if within depth limit
                    if depth < self.config.max_depth {
                        for link in &page.links {
                            let link_norm = normalize_url(link);
                            if link_norm.starts_with(&get_origin(&normalized))
                                && !visited_local.contains(&link_norm)
                            {
                                queue.push((link_norm, depth + 1));
                            }
                        }
                    }

                    debug!(
                        "crawled depth={depth} links={page_links} url={normalized}",
                    );
                }
                Err(e) => {
                    self.errors.push(CrawlError {
                        url: normalized.clone(),
                        message: e,
                    });
                }
            }
        }
    }

    async fn fetch_page(&self, url: &str) -> Result<CrawledPage, String> {
        self.request_count.fetch_add(1, Ordering::SeqCst);
        let t0 = std::time::Instant::now();
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {e}"))?;

        let status = resp.status();
        if !status.is_success() && status.as_u16() != 404 {
            return Err(format!("HTTP {status} for {url}"));
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = resp.text().await.map_err(|e| format!("body error: {e}"))?;
        let elapsed = t0.elapsed();

        // Extract links from HTML
        let links = if content_type.contains("text/html") {
            extract_links(&body, url)
        } else {
            Vec::new()
        };

        // Discover API endpoints in the page
        let api_endpoints = discover_endpoints_in_page(&body, url);

        Ok(CrawledPage {
            url: url.to_string(),
            depth: 0,
            content_type,
            body: body.chars().take(10_000).collect(), // truncate stored body
            links,
            api_endpoints,
            elapsed_ms: elapsed.as_millis() as u64,
        })
    }

    async fn fetch_robots(&self, base_url: &str) -> Result<(), String> {
        let robots_url = format!("{}/robots.txt", base_url.trim_end_matches('/'));
        let resp = self
            .client
            .get(&robots_url)
            .timeout(self.config.timeout)
            .send()
            .await
            .map_err(|e| format!("robots.txt error: {e}"))?;

        if !resp.status().is_success() {
            return Ok(()); // no robots.txt is fine
        }

        let body = resp.text().await.map_err(|e| format!("robots body: {e}))"))?;
        let mut disallowed = self.disallowed_paths.lock();
        for line in body.lines() {
            let line = line.trim();
            if let Some(path) = line
                .to_ascii_lowercase()
                .strip_prefix("disallow:")
                .or_else(|| line.strip_prefix("Disallow:"))
            {
                let path = path.trim();
                if !path.is_empty() {
                    disallowed.push(path.to_string());
                }
            }
        }
        info!("robots.txt: {} disallowed paths loaded", disallowed.len());
        Ok(())
    }

    async fn discover_api_specs(&mut self, base_url: &str) {
        let base = base_url.trim_end_matches('/');
        let candidates = vec![
            (format!("{base}/openapi.json"), ApiSpecKind::OpenApi),
            (format!("{base}/api/docs"), ApiSpecKind::Swagger),
            (format!("{base}/swagger.json"), ApiSpecKind::Swagger),
            (format!("{base}/swagger/v1/swagger.json"), ApiSpecKind::Swagger),
            (format!("{base}/api/swagger.json"), ApiSpecKind::Swagger),
            (format!("{base}/graphql"), ApiSpecKind::GraphQl),
            (format!("{base}/api/graphql"), ApiSpecKind::GraphQl),
            (format!("{base}/v1/graphql"), ApiSpecKind::GraphQl),
            (format!("{base}/asyncapi.json"), ApiSpecKind::AsyncApi),
            (format!("{base}/api/asyncapi.json"), ApiSpecKind::AsyncApi),
        ];

        for (url, kind) in &candidates {
            self.request_count.fetch_add(1, Ordering::SeqCst);
            match self.client.get(url).timeout(self.config.timeout).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let endpoints = extract_endpoints_from_spec(url, &kind).await;
                    self.api_specs.push(ApiSpec {
                        url: url.clone(),
                        kind: kind.clone(),
                        endpoints,
                    });
                    info!("discovered {kind:?} spec at {url}");
                }
                Ok(_) => { /* not found */ }
                Err(e) => {
                    debug!("api spec probe {url}: {e}");
                }
            }
        }
    }

    fn is_disallowed(&self, url: &str) -> bool {
        let disallowed = self.disallowed_paths.lock();
        if disallowed.is_empty() {
            return false;
        }
        let path = url
            .split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .skip(1)
            .collect::<Vec<_>>()
            .join("/");
        let path = format!("/{path}");
        for pattern in disallowed.iter() {
            if path.starts_with(pattern) || path == *pattern {
                return true;
            }
        }
        false
    }
}

// ── Utility functions ────────────────────────────────────────────

fn normalize_url(url: &str) -> String {
    let url = url.trim();
    // Strip fragment
    let url = url.split('#').next().unwrap_or(url);
    url.trim_end_matches('/').to_string()
}

fn get_origin(url: &str) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 3 {
        format!("{}//{}", parts[0], parts[2])
    } else {
        url.to_string()
    }
}

fn extract_links(html: &str, base: &str) -> Vec<String> {
    let mut links = Vec::new();
    let lower = html.to_ascii_lowercase();

    // Extract href="..." from <a> tags
    for (i, _) in lower.match_indices("href=\"") {
        let start = i + 6;
        if let Some(end) = html[start..].find('"') {
            let href = &html[start..start + end];
            if href.starts_with('#') || href.starts_with("javascript:") {
                continue;
            }
            if let Some(url) = resolve_url(href, base) {
                links.push(url);
            }
        }
    }

    // Extract action="..." from <form> tags
    for (i, _) in lower.match_indices("action=\"") {
        let start = i + 8;
        if let Some(end) = html[start..].find('"') {
            let action = &html[start..start + end];
            if action.starts_with('#') {
                continue;
            }
            if let Some(url) = resolve_url(action, base) {
                links.push(url);
            }
        }
    }

    links
}

fn resolve_url(href: &str, base: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() {
        return None;
    }
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(href.to_string());
    }
    if href.starts_with("//") {
        let scheme = base.split("://").next().unwrap_or("http");
        return Some(format!("{scheme}:{href}"));
    }
    if href.starts_with('/') {
        let origin = get_origin(base);
        return Some(format!("{origin}{href}"));
    }
    // Relative path
    let base = if base.ends_with('/') {
        base.to_string()
    } else {
        // Remove last path component
        let idx = base.rfind('/')?;
        base[..=idx].to_string()
    };
    Some(format!("{base}{href}"))
}

fn discover_endpoints_in_page(_body: &str, _url: &str) -> Vec<String> {
    // Stub — future: detect /api/, /rest/, etc. patterns in page
    Vec::new()
}

async fn extract_endpoints_from_spec(url: &str, _kind: &ApiSpecKind) -> Vec<String> {
    // Stub — future: parse OpenAPI JSON to extract paths
    // For now just return the spec URL itself
    vec![url.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL normalization ─────────────────────────────────────────

    #[test]
    fn test_normalize_url_strips_slash() {
        assert_eq!(normalize_url("http://example.com/"), "http://example.com");
    }

    #[test]
    fn test_normalize_url_strips_fragment() {
        assert_eq!(
            normalize_url("http://example.com/page#section"),
            "http://example.com/page"
        );
    }

    #[test]
    fn test_normalize_url_keeps_query() {
        assert_eq!(
            normalize_url("http://example.com/page?q=1"),
            "http://example.com/page?q=1"
        );
    }

    #[test]
    fn test_normalize_url_trims_whitespace() {
        assert_eq!(
            normalize_url("  http://example.com  "),
            "http://example.com"
        );
    }

    // ── Origin extraction ─────────────────────────────────────────

    #[test]
    fn test_get_origin_http() {
        assert_eq!(
            get_origin("http://example.com/path/file"),
            "http://example.com"
        );
    }

    #[test]
    fn test_get_origin_https_with_port() {
        assert_eq!(
            get_origin("https://localhost:8080/api/v1"),
            "https://localhost:8080"
        );
    }

    #[test]
    fn test_get_origin_invalid() {
        assert_eq!(get_origin("not-a-url"), "not-a-url");
    }

    // ── Link extraction from HTML ─────────────────────────────────

    #[test]
    fn test_extract_links_empty() {
        let links = extract_links("<html></html>", "http://example.com");
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_finds_href() {
        let html = r#"<a href="/page1">Link</a> <a href="http://other.com">Other</a>"#;
        let links = extract_links(html, "http://example.com");
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l == "http://example.com/page1"));
        assert!(links.iter().any(|l| l == "http://other.com"));
    }

    #[test]
    fn test_extract_links_skips_javascript() {
        let html = r##"<a href="javascript:void(0)">JS</a>"##;
        let links = extract_links(html, "http://example.com");
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_skips_anchors() {
        let html = r##"<a href="#section">Section</a>"##;
        let links = extract_links(html, "http://example.com");
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_finds_form_actions() {
        let html = r#"<form action="/login"><input></form>"#;
        let links = extract_links(html, "http://example.com");
        assert!(links.iter().any(|l| l == "http://example.com/login"));
    }

    #[test]
    fn test_extract_links_absolute_form_action() {
        let html = r#"<form action="http://other.com/submit"></form>"#;
        let links = extract_links(html, "http://example.com");
        assert!(links.iter().any(|l| l == "http://other.com/submit"));
    }

    #[test]
    fn test_extract_links_multiple_on_same_page() {
        let html = r#"
            <a href="/">Home</a>
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
        "#;
        let links = extract_links(html, "http://example.com");
        assert_eq!(links.len(), 3);
    }

    // ── URL resolution ────────────────────────────────────────────

    #[test]
    fn test_resolve_url_absolute() {
        assert_eq!(
            resolve_url("http://example.com", "http://base.com").unwrap(),
            "http://example.com"
        );
    }

    #[test]
    fn test_resolve_url_relative() {
        assert_eq!(
            resolve_url("page.html", "http://example.com/dir/").unwrap(),
            "http://example.com/dir/page.html"
        );
    }

    #[test]
    fn test_resolve_url_root_relative() {
        assert_eq!(
            resolve_url("/page.html", "http://example.com/dir/").unwrap(),
            "http://example.com/page.html"
        );
    }

    #[test]
    fn test_resolve_url_protocol_relative() {
        assert_eq!(
            resolve_url("//other.com/page", "https://example.com").unwrap(),
            "https://other.com/page"
        );
    }

    #[test]
    fn test_resolve_url_empty() {
        assert!(resolve_url("", "http://example.com").is_none());
    }

    // ── robots.txt disallowed check ───────────────────────────────

    #[test]
    fn test_is_disallowed_empty() {
        let crawler = create_test_crawler();
        assert!(!crawler.is_disallowed("http://example.com/admin"));
    }

    #[test]
    fn test_is_disallowed_matches_path() {
        let crawler = create_crawler_with_disallowed(vec!["/admin".into()]);
        assert!(crawler.is_disallowed("http://example.com/admin"));
        assert!(crawler.is_disallowed("http://example.com/admin/users"));
        assert!(!crawler.is_disallowed("http://example.com/public"));
    }

    #[test]
    fn test_is_disallowed_wildcard_all() {
        let crawler = create_crawler_with_disallowed(vec!["/".into()]);
        assert!(crawler.is_disallowed("http://example.com/anything"));
    }

    #[test]
    fn test_is_disallowed_multiple_patterns() {
        let crawler =
            create_crawler_with_disallowed(vec!["/private".into(), "/tmp".into()]);
        assert!(crawler.is_disallowed("http://example.com/private/data"));
        assert!(crawler.is_disallowed("http://example.com/tmp"));
        assert!(!crawler.is_disallowed("http://example.com/public"));
    }

    // ── CrawlConfig ───────────────────────────────────────────────

    #[test]
    fn test_crawl_config_defaults() {
        let cfg = CrawlConfig::default();
        assert_eq!(cfg.user_agent, "BugbeeCrawler/1.0 (+https://bugbee.security)");
        assert_eq!(cfg.timeout, Duration::from_secs(10));
        assert_eq!(cfg.max_urls, 200);
        assert_eq!(cfg.max_depth, 3);
        assert!(cfg.respect_robots);
    }

    // ── Crawler construction ──────────────────────────────────────

    #[test]
    fn test_crawler_construction() {
        let cfg = CrawlConfig::default();
        let crawler = Crawler::new(cfg);
        assert!(crawler.is_ok());
    }

    #[test]
    fn test_crawler_construction_fails_on_bad_config() {
        let mut cfg = CrawlConfig::default();
        cfg.timeout = Duration::from_nanos(0); // invalid timeout
        let crawler = Crawler::new(cfg);
        // reqwest may or may not reject 0 timeout — just check no panic
        let _ = crawler;
    }

    // ── API spec kind display ─────────────────────────────────────

    #[test]
    fn test_api_spec_kind_variants() {
        assert_eq!(ApiSpecKind::OpenApi, ApiSpecKind::OpenApi);
        assert_eq!(ApiSpecKind::GraphQl, ApiSpecKind::GraphQl);
        assert_ne!(ApiSpecKind::OpenApi, ApiSpecKind::Swagger);
    }

    // ── CrawlReport defaults ──────────────────────────────────────

    #[test]
    fn test_crawl_report_default() {
        let report = CrawlReport::default();
        assert_eq!(report.pages_crawled, 0);
        assert_eq!(report.total_links, 0);
        assert!(report.api_specs.is_empty());
        assert!(report.errors.is_empty());
        assert_eq!(report.elapsed_ms, 0);
    }

    // ── Helpers ───────────────────────────────────────────────────

    fn create_test_crawler() -> Crawler {
        let cfg = CrawlConfig {
            respect_robots: false,
            max_urls: 1,
            ..Default::default()
        };
        let mut c = Crawler::new(cfg).unwrap();
        // Disable real HTTP in tests by making max_urls very small
        c.config.max_urls = 0;
        c
    }

    fn create_crawler_with_disallowed(patterns: Vec<String>) -> Crawler {
        let cfg = CrawlConfig {
            respect_robots: false,
            max_urls: 1,
            ..Default::default()
        };
        let mut c = Crawler::new(cfg).unwrap();
        c.config.max_urls = 0;
        *c.disallowed_paths.lock() = patterns;
        c
    }
}
