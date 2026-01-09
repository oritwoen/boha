use super::playwright::{PlaywrightContext, Result, TweetArchive};
use playwright::api::frame::FrameState;
use playwright::api::DocumentLoadState;
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

const WAYBACK_API_ENDPOINT: &str = "https://archive.org/wayback/available";

// 10-15 req/min => 4-6 seconds between requests. Pick 5s (~12/min) to stay safe.
const MIN_REQUEST_INTERVAL: Duration = Duration::from_secs(5);

const MAX_ATTEMPTS: usize = 5;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static NEXT_ALLOWED_REQUEST: OnceLock<Mutex<Instant>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct WaybackAvailableResponse {
    archived_snapshots: Option<WaybackArchivedSnapshots>,
}

#[derive(Debug, Deserialize)]
struct WaybackArchivedSnapshots {
    closest: Option<WaybackClosestSnapshot>,
}

#[derive(Debug, Deserialize)]
struct WaybackClosestSnapshot {
    available: Option<bool>,
    url: Option<String>,
    status: Option<String>,
}

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("boha-scripts/0.1 (+https://github.com/oritwoen/boha)")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client")
    })
}

fn limiter_state() -> &'static Mutex<Instant> {
    NEXT_ALLOWED_REQUEST.get_or_init(|| Mutex::new(Instant::now()))
}

async fn rate_limit() {
    let mut next_allowed = limiter_state().lock().await;
    let now = Instant::now();

    if *next_allowed > now {
        tokio::time::sleep(*next_allowed - now).await;
    }

    *next_allowed = Instant::now() + MIN_REQUEST_INTERVAL;
}

fn jitter_ms(max_ms: u64) -> Duration {
    // Avoid adding a rand dependency; derive some entropy from current time.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);

    Duration::from_millis(nanos % (max_ms + 1))
}

async fn backoff(attempt: usize, retry_after: Option<Duration>) {
    // Base backoff: 1s, 2s, 4s, 8s ... capped.
    let exp = 1u64 << attempt.min(4);
    let exp_delay = Duration::from_secs(exp);

    let delay = retry_after.unwrap_or(exp_delay).max(MIN_REQUEST_INTERVAL) + jitter_ms(250);

    tokio::time::sleep(delay).await;
}

fn is_http_2xx(status: &str) -> bool {
    let code = match status.trim().parse::<u16>() {
        Ok(x) => x,
        Err(_) => return false,
    };

    (200..300).contains(&code)
}

/// Check if a URL has a valid Wayback Machine snapshot.
///
/// Returns `Ok(Some(wayback_url))` only when:
/// - `archived_snapshots.closest.available == true`
/// - `closest.status` exists, is a *string*, and parses to HTTP 2xx
/// - `closest.url` exists
///
/// Returns `Ok(None)` when there is no snapshot (including `archived_snapshots: {}`).
pub async fn check_availability(url: &str) -> Result<Option<String>> {
    if url.trim().is_empty() {
        return Err(io_err("check_availability: url is empty"));
    }

    let request_url = reqwest::Url::parse_with_params(WAYBACK_API_ENDPOINT, [("url", url)])?;

    for attempt in 0..MAX_ATTEMPTS {
        rate_limit().await;

        let response = match http_client().get(request_url.clone()).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("    Wayback API request error: {e}");
                if attempt + 1 < MAX_ATTEMPTS {
                    backoff(attempt, None).await;
                    continue;
                }
                return Err(e.into());
            }
        };

        if response.status().as_u16() == 429 {
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(Duration::from_secs);

            if attempt + 1 < MAX_ATTEMPTS {
                eprintln!("    Wayback API rate limited (429), backing off...");
                backoff(attempt, retry_after).await;
                continue;
            }

            return Err(io_err("Wayback API rate limited after retries"));
        }

        if !response.status().is_success() {
            if response.status().is_server_error() && attempt + 1 < MAX_ATTEMPTS {
                eprintln!(
                    "    Wayback API server error {}, retrying...",
                    response.status()
                );
                backoff(attempt, None).await;
                continue;
            }

            return Err(io_err(format!("Wayback API error: {}", response.status())));
        }

        let body: WaybackAvailableResponse = response.json().await?;

        let closest = body
            .archived_snapshots
            .and_then(|s| s.closest)
            .filter(|c| c.available.unwrap_or(false))
            .filter(|c| c.status.as_deref().is_some_and(is_http_2xx));

        return Ok(closest.and_then(|c| c.url));
    }

    Err(io_err("Wayback API: unreachable"))
}

/// Screenshot a Wayback-archived tweet page and extract metadata.
///
/// This is best-effort and intentionally conservative: if the archived page is too broken to
/// locate any reasonable tweet text, return an error (don't panic).
pub async fn screenshot_wayback(
    ctx: &PlaywrightContext,
    wayback_url: &str,
) -> Result<TweetArchive> {
    if wayback_url.trim().is_empty() {
        return Err(io_err("screenshot_wayback: wayback_url is empty"));
    }

    // Requests to web.archive.org can also be rate limited.
    rate_limit().await;

    let page = ctx.new_page().await?;

    page.goto_builder(wayback_url)
        .wait_until(DocumentLoadState::DomContentLoaded)
        .goto()
        .await
        .map_err(|e| pw_err("page.goto(wayback)", e))?;

    let tweet = find_tweet_container(&page).await;

    let text = extract_tweet_text(&page, tweet.as_ref()).await?;
    if text.trim().is_empty() {
        return Err(io_err("Wayback tweet text not found"));
    }

    let mut author = extract_author_from_dom(&page).await.unwrap_or_default();
    if author.is_empty() {
        author = extract_author_from_url(wayback_url)
            .unwrap_or_else(|| {
                eprintln!("Warning: failed to extract author for {wayback_url}");
                "unknown".to_string()
            })
            .to_string();
    }

    let date = extract_date_yyyy_mm_dd(&page).await.unwrap_or_else(|| {
        eprintln!("Warning: failed to extract date for {wayback_url}");
        "unknown".to_string()
    });

    let png = match tweet {
        Some(el) => el
            .screenshot_builder()
            .await
            .screenshot()
            .await
            .map_err(|e| pw_err("wayback tweet.screenshot", e))?,
        None => screenshot_full_page(&page).await?,
    };

    // Best effort; don't block the caller if page close fails.
    let _ = page.close(None).await;

    Ok(TweetArchive {
        text,
        author,
        date,
        png,
    })
}

async fn find_tweet_container(
    page: &playwright::api::Page,
) -> Option<playwright::api::ElementHandle> {
    // Try modern X layout first.
    if let Ok(Some(el)) = page
        .wait_for_selector_builder("article[data-testid=\"tweet\"]")
        .state(FrameState::Visible)
        .timeout(10_000.0)
        .wait_for_selector()
        .await
    {
        return Some(el);
    }

    // Older Twitter layouts (Wayback snapshots are often from 2015-2016).
    for selector in [
        "div.permalink-tweet",
        "div.permalink-tweet-container",
        "div.tweet",
        "article",
    ] {
        if let Ok(Some(el)) = page.query_selector(selector).await {
            return Some(el);
        }
    }

    None
}

async fn extract_tweet_text(
    page: &playwright::api::Page,
    tweet: Option<&playwright::api::ElementHandle>,
) -> Result<String> {
    // Modern selector.
    if let Ok(Some(el)) = page.query_selector("[data-testid=\"tweetText\"]").await {
        return Ok(el
            .inner_text()
            .await
            .map_err(|e| pw_err("tweetText.inner_text", e))?);
    }

    // Common older selectors.
    for selector in [".tweet-text", ".TweetTextSize"] {
        if let Ok(Some(el)) = page.query_selector(selector).await {
            return Ok(el
                .inner_text()
                .await
                .map_err(|e| pw_err("tweet legacy text.inner_text", e))?);
        }
    }

    // Try <p> inside tweet-ish containers.
    for selector in [
        "article[data-testid=\"tweet\"] p",
        "div.permalink-tweet p",
        "div.permalink-tweet-container p",
        "div.tweet p",
    ] {
        if let Ok(Some(el)) = page.query_selector(selector).await {
            let t = el
                .inner_text()
                .await
                .map_err(|e| pw_err("tweet p.inner_text", e))?;
            if !t.trim().is_empty() {
                return Ok(t);
            }
        }
    }

    // Ultimate fallback: whatever is visible on the page.
    if let Ok(Some(body)) = page.query_selector("body").await {
        return Ok(body
            .inner_text()
            .await
            .map_err(|e| pw_err("body.inner_text", e))?);
    }

    // Best effort: if we at least have a tweet container, return its text.
    if let Some(tweet) = tweet {
        let t = tweet
            .inner_text()
            .await
            .map_err(|e| pw_err("tweet_container.inner_text", e))?;
        return Ok(t);
    }

    Ok(String::new())
}

async fn screenshot_full_page(page: &playwright::api::Page) -> Result<Vec<u8>> {
    // Prefer body element screenshot; avoids Playwright full-page quirks.
    if let Ok(Some(body)) = page.query_selector("body").await {
        return Ok(body
            .screenshot_builder()
            .await
            .screenshot()
            .await
            .map_err(|e| pw_err("body.screenshot", e))?);
    }

    page.screenshot_builder()
        .screenshot()
        .await
        .map_err(|e| pw_err("page.screenshot", e))
}

async fn extract_author_from_dom(page: &playwright::api::Page) -> Option<String> {
    // Modern X selector (same as live).
    if let Ok(Some(el)) = page
        .query_selector("[data-testid=\"User-Name\"] a[role=\"link\"]")
        .await
    {
        let text = el.inner_text().await.ok()?;
        let handle = text
            .split_whitespace()
            .find(|p| p.starts_with('@'))
            .map(|p| p.trim().trim_end_matches(':').to_string())?;
        return Some(handle);
    }

    // Older Twitter layout: <span class="username">@handle</span>
    for selector in ["span.username", "span.screen-name", "span.nickname"] {
        if let Ok(Some(el)) = page.query_selector(selector).await {
            let text = el.inner_text().await.ok()?;
            let handle = text
                .split_whitespace()
                .find(|p| p.starts_with('@'))
                .map(|p| p.trim().trim_end_matches(':').to_string())?;
            return Some(handle);
        }
    }

    None
}

fn extract_author_from_url(url: &str) -> Option<String> {
    // Wayback format: https://web.archive.org/web/<timestamp>/<original_url>
    // Original URL may be https://twitter.com/<user>/status/<id> or https://x.com/<user>/status/<id>
    let embedded = url.split("/http").nth(1).map(|s| format!("http{s}"));
    let candidate = embedded.as_deref().unwrap_or(url);

    let without_scheme = candidate.split("//").nth(1).unwrap_or(candidate);
    let mut parts = without_scheme.split('/');
    let _host = parts.next()?;
    let username = parts.next()?;
    if username.is_empty() {
        return None;
    }

    Some(format!("@{}", username))
}

async fn extract_date_yyyy_mm_dd(page: &playwright::api::Page) -> Option<String> {
    let el = page.query_selector("time[datetime]").await.ok()??;
    let dt = el.get_attribute("datetime").await.ok()??;
    if dt.len() >= 10 {
        Some(dt[..10].to_string())
    } else {
        None
    }
}

fn io_err(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    std::io::Error::new(std::io::ErrorKind::Other, msg.into()).into()
}

fn pw_err<E: std::fmt::Debug>(context: &'static str, e: E) -> Box<dyn std::error::Error> {
    io_err(format!("{context}: {e:?}"))
}
