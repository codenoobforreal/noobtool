use fantoccini::{Client, ClientBuilder};
use serde_json::{Value, json, map};

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Session error: {0}")]
    Session(#[from] fantoccini::error::NewSessionError),
    #[error("Command error: {0}")]
    Cmd(#[from] fantoccini::error::CmdError),
    #[error("type conversion error: {0}")]
    Conversion(String),
}

pub struct DriverClient {
    pub inner: Client,
}

impl DriverClient {
    pub async fn connect(url: &str, headless: bool) -> Result<Self, DriverError> {
        // Create webdriver client
        //
        // ## preconditions
        // a WebDriver compatible process running on specified port usually 4444.
        //
        // ## download chromedriver
        // go to https://googlechromelabs.github.io/chrome-for-testing/ and save chromedriver to C:\Windows\chromedriver.xe.
        //
        // ## commands
        // ```shell
        // chromedriver --port=4444
        // chromedriver -h
        // ```
        //
        // ## capabilities
        // https://peter.sh/experiments/chromium-command-line-switches/
        //
        // ## security
        // https://developer.chrome.com/docs/chromedriver/security-considerations
        let inner = ClientBuilder::native()
            .capabilities({
                let mut caps = map::Map::new();
                if headless {
                    caps.insert(
                        "goog:chromeOptions".to_string(),
                        json!({
                            "args": [
                                "--headless",
                            ],
                        }),
                    );
                }
                caps
            })
            .connect(url)
            .await?;

        Ok(Self { inner })
    }

    pub async fn close(self) -> Result<(), DriverError> {
        self.inner.close().await?;
        Ok(())
    }

    pub async fn scroll_to_bottom(&self, height: usize) -> Result<(), DriverError> {
        self.inner
            .execute(&format!("window.scrollTo(0, {height});"), vec![])
            .await?;

        Ok(())
    }

    pub async fn scroll_height(&self) -> Result<u64, DriverError> {
        let script_res = self
            .inner
            .execute("return document.body.scrollHeight;", [].to_vec())
            .await?;

        script_res
            .as_u64()
            .ok_or_else(|| DriverError::Conversion("can't convert scrollHeight".to_string()))
    }

    pub async fn smooth_scroll_to_bottom(&self, duration_ms: usize) -> Result<(), DriverError> {
        static SCROLL_SCRIPT: &str = r#"
            return new Promise((resolve) => {
                const startY = window.scrollY;
                const targetY = document.body.scrollHeight;
                const duration = arguments[0];
                let startTime = null;

                function step(timestamp) {
                    if (!startTime) startTime = timestamp;
                    const elapsed = timestamp - startTime;
                    const progress = Math.min(elapsed / duration, 1);
                    const ease = progress => progress < 0.5
                        ? 2 * progress * progress
                        : -1 + (4 - 2 * progress) * progress;

                    window.scrollTo(0, startY + (targetY - startY) * ease(progress));

                    if (elapsed < duration) {
                        requestAnimationFrame(step);
                    } else {
                        resolve();
                    }
                }
                requestAnimationFrame(step);
            });
        "#;

        let params: [Value; 1] = [duration_ms.into()];
        self.inner.execute(SCROLL_SCRIPT, params.to_vec()).await?;

        Ok(())
    }
}
