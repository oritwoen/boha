use playwright::api::frame::FrameState;
use playwright::api::{Browser, BrowserContext, DocumentLoadState, StorageState, Viewport};
use playwright::Playwright;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const VIEWPORT_WIDTH: i32 = 1280;
const VIEWPORT_HEIGHT: i32 = 720;

#[derive(Debug, Clone)]
pub struct TweetArchive {
    pub text: String,
    pub author: String,
    pub date: String,
    pub png: Vec<u8>,
}

pub struct PlaywrightContext {
    #[allow(dead_code)]
    playwright: Playwright,
    #[allow(dead_code)]
    browser: Browser,
    context: BrowserContext,
    state_path: PathBuf,
}

impl PlaywrightContext {
    /// Initialize Playwright and an authenticated browser context.
    ///
    /// This uses a single `storage_state` JSON file (cookies + local storage) for persistence.
    /// If the file is missing or invalid, it launches a headed browser and waits for manual login.
    pub async fn new(state_path: &str) -> Result<Self> {
        let state_path = PathBuf::from(state_path);

        let playwright = Playwright::initialize()
            .await
            .map_err(|e| pw_err("Playwright::initialize", e))?;

        // Ensure the bundled browsers exist (first-time setup).
        playwright
            .install_chromium()
            .map_err(|e| io_err(format!("playwright install chromium failed: {e}")))?;

        let storage_state = load_storage_state(&state_path);

        let headed = storage_state.is_none();
        let browser = playwright
            .chromium()
            .launcher()
            .headless(!headed)
            .launch()
            .await
            .map_err(|e| pw_err("chromium.launch", e))?;

        let mut context_builder = browser
            .context_builder()
            .viewport(Some(Viewport {
                width: VIEWPORT_WIDTH,
                height: VIEWPORT_HEIGHT,
            }))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36");

        if let Some(state) = storage_state {
            context_builder = context_builder.storage_state(state);
        }

        let context = context_builder
            .build()
            .await
            .map_err(|e| pw_err("browser.new_context", e))?;

        context
            .add_init_script(
                "Object.defineProperty(navigator, 'webdriver', {get: () => undefined})",
            )
            .await
            .map_err(|e| pw_err("context.add_init_script", e))?;

        let this = Self {
            playwright,
            browser,
            context,
            state_path,
        };

        if headed {
            if let Err(e) = this.load_cookies_from_file().await {
                eprintln!("Warning: failed to load cookies from file: {}", e);
                eprintln!("Falling back to manual login...");
                this.run_manual_login().await?;
            } else {
                eprintln!("Loaded cookies from scripts/twitter-cookies.json");
                this.save_current_state().await?;
            }
        }

        Ok(this)
    }

    pub(crate) async fn new_page(&self) -> Result<playwright::api::Page> {
        self.context
            .new_page()
            .await
            .map_err(|e| pw_err("context.new_page", e))
    }

    /// Navigate to an X/Twitter tweet URL and capture metadata + a PNG screenshot.
    ///
    /// Retry policy: 3 attempts with exponential backoff (100ms, 200ms, 400ms).
    pub async fn screenshot_tweet(&self, url: &str) -> Result<TweetArchive> {
        let mut delay_ms = 100u64;
        for attempt in 0..3 {
            if attempt > 0 {
                eprintln!("    Retry {}/3 after {}ms...", attempt + 1, delay_ms);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms *= 2;
            }

            match self.screenshot_tweet_once(url).await {
                Ok(x) => return Ok(x),
                Err(e) if attempt < 2 => {
                    eprintln!("    Attempt {} failed: {}", attempt + 1, e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err("unreachable".into())
    }

    async fn load_cookies_from_file(&self) -> Result<()> {
        let cookies_path = self
            .state_path
            .parent()
            .ok_or("invalid state path")?
            .join("twitter-cookies.json");

        if !cookies_path.exists() {
            return Err(format!("Cookies file not found: {}", cookies_path.display()).into());
        }

        let content = std::fs::read_to_string(&cookies_path)?;
        let mut cookies: Vec<playwright::api::Cookie> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse cookies JSON: {}", e))?;

        let mut x_com_cookies = Vec::new();
        for cookie in &cookies {
            if cookie.domain == Some(".twitter.com".to_string()) {
                let mut x_cookie = cookie.clone();
                x_cookie.domain = Some(".x.com".to_string());
                x_com_cookies.push(x_cookie);
            }
        }
        cookies.extend(x_com_cookies);

        self.context
            .add_cookies(&cookies)
            .await
            .map_err(|e| pw_err("context.add_cookies", e))?;

        Ok(())
    }

    async fn save_current_state(&self) -> Result<()> {
        let storage_state = self
            .context
            .storage_state()
            .await
            .map_err(|e| pw_err("context.storage_state", e))?;

        save_storage_state(&self.state_path, &storage_state)?;
        eprintln!("Saved storage state to {}", self.state_path.display());

        Ok(())
    }

    async fn run_manual_login(&self) -> Result<()> {
        eprintln!(
            "No valid storage state found at {}",
            self.state_path.display()
        );
        eprintln!("Launching headed browser for manual X login...");

        let page = self
            .context
            .new_page()
            .await
            .map_err(|e| pw_err("context.new_page", e))?;

        page.goto_builder("https://x.com/login")
            .wait_until(DocumentLoadState::DomContentLoaded)
            .goto()
            .await
            .map_err(|e| pw_err("page.goto(x.com/login)", e))?;

        eprintln!(
            "Please complete login in the opened browser, then press Enter here to continue."
        );

        // Flush stderr so the prompt shows up even if output is buffered.
        std::io::stderr().flush().ok();

        let mut _input = String::new();
        std::io::stdin().read_line(&mut _input)?;

        let storage_state = self
            .context
            .storage_state()
            .await
            .map_err(|e| pw_err("context.storage_state", e))?;

        save_storage_state(&self.state_path, &storage_state)?;
        eprintln!("Saved storage state to {}", self.state_path.display());

        Ok(())
    }

    async fn screenshot_tweet_once(&self, url: &str) -> Result<TweetArchive> {
        let page = self
            .context
            .new_page()
            .await
            .map_err(|e| pw_err("context.new_page", e))?;

        page.goto_builder(url)
            .wait_until(DocumentLoadState::NetworkIdle)
            .goto()
            .await
            .map_err(|e| pw_err("page.goto(tweet)", e))?;

        let tweet = page
            .wait_for_selector_builder("article[data-testid=\"tweet\"]")
            .state(FrameState::Visible)
            .timeout(10_000.0)
            .wait_for_selector()
            .await
            .map_err(|e| pw_err("wait_for tweet container", e))?
            .ok_or_else(|| io_err("tweet container not found"))?;

        tweet
            .scroll_into_view_if_needed(None)
            .await
            .map_err(|e| pw_err("tweet.scroll_into_view", e))?;

        tokio::time::sleep(Duration::from_millis(2000)).await;

        let _ = page
            .evaluate::<(), ()>(
                r#"() => {
                document.querySelector('[role="banner"]')?.remove();
                document.querySelector('[data-testid="TopNavBar"]')?.remove();
                
                const article = document.querySelector('article[data-testid="tweet"]');
                if (article) {
                    const header = article.querySelector('[role="button"][aria-label*="Back"], [role="button"][aria-label*="Wstecz"]');
                    if (header) {
                        header.closest('div[style*="position: sticky"], div[style*="position: fixed"]')?.remove();
                        header.parentElement?.remove();
                    }
                    
                    article.querySelectorAll('div').forEach(div => {
                        const style = window.getComputedStyle(div);
                        if (style.background && style.background.includes('gradient') && 
                            div.getBoundingClientRect().top < 50) {
                            div.remove();
                        }
                    });
                }
            }"#,
                (),
            )
            .await;

        // The text container is absent for media-only tweets; that's ok.
        let text = match page.query_selector("[data-testid=\"tweetText\"]").await {
            Ok(Some(el)) => el
                .inner_text()
                .await
                .map_err(|e| pw_err("tweetText.inner_text", e))?,
            _ => String::new(),
        };

        let mut author = extract_author_from_dom(&page).await.unwrap_or_default();
        if author.is_empty() {
            author = extract_author_from_url(url)
                .unwrap_or_else(|| {
                    eprintln!("Warning: failed to extract author for {}", url);
                    "unknown".to_string()
                })
                .to_string();
        }

        let date = extract_date_yyyy_mm_dd(&page).await.unwrap_or_else(|| {
            eprintln!("Warning: failed to extract date for {}", url);
            "unknown".to_string()
        });

        let png = tweet
            .screenshot_builder()
            .await
            .screenshot()
            .await
            .map_err(|e| pw_err("tweet.screenshot", e))?;

        // Best effort; don't block the caller if page close fails.
        let _ = page.close(None).await;

        Ok(TweetArchive {
            text,
            author,
            date,
            png,
        })
    }
}

fn load_storage_state(path: &Path) -> Option<StorageState> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_storage_state(path: &Path, state: &StorageState) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(state)?;
    std::fs::write(path, content)?;
    Ok(())
}

async fn extract_author_from_dom(page: &playwright::api::Page) -> Option<String> {
    let el = page
        .query_selector("[data-testid=\"User-Name\"] a[role=\"link\"]")
        .await
        .ok()??;

    let text = el.inner_text().await.ok()?;
    let handle = text
        .split_whitespace()
        .find(|p| p.starts_with('@'))
        .map(|p| p.trim().trim_end_matches(':').to_string())?;

    Some(handle)
}

fn extract_author_from_url(url: &str) -> Option<String> {
    // Expected: https://x.com/<username>/status/<id>
    let without_scheme = url.split("//").nth(1).unwrap_or(url);
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
