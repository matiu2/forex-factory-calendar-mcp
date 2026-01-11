use chrono::{Datelike, NaiveDate};
use color_eyre::{Result, eyre::eyre};
use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;
use tracing::{debug, info};

/// Fetches the raw HTML content from Forex Factory calendar page.
pub struct CalendarFetcher {
    browser: Browser,
}

impl CalendarFetcher {
    /// Create a new fetcher with a headless Chrome browser.
    pub fn new() -> Result<Self> {
        info!("Launching headless Chrome browser...");

        let launch_options = LaunchOptions {
            headless: true,
            sandbox: true,
            idle_browser_timeout: Duration::from_secs(300),
            ..Default::default()
        };

        let browser =
            Browser::new(launch_options).map_err(|e| eyre!("Failed to launch browser: {e}"))?;

        Ok(Self { browser })
    }

    /// Fetch calendar HTML for a specific week.
    /// The `week` parameter should be in format like "jun1.2025" for the week of June 1, 2025.
    pub fn fetch_week(&self, week: &str) -> Result<String> {
        let url = format!("https://www.forexfactory.com/calendar?week={week}");
        self.fetch_url(&url)
    }

    /// Fetch calendar HTML for a date range.
    /// Forex Factory uses week-based navigation, so we'll fetch the week containing the start date.
    pub fn fetch_date(&self, date: NaiveDate) -> Result<String> {
        let week = format_week_param(date);
        self.fetch_week(&week)
    }

    /// Fetch calendar HTML for today.
    pub fn fetch_today(&self) -> Result<String> {
        self.fetch_url("https://www.forexfactory.com/calendar?day=today")
    }

    /// Fetch calendar HTML for this week.
    pub fn fetch_this_week(&self) -> Result<String> {
        self.fetch_url("https://www.forexfactory.com/calendar?week=this")
    }

    /// Fetch the raw HTML from a URL.
    fn fetch_url(&self, url: &str) -> Result<String> {
        info!("Fetching calendar from: {url}");

        let tab = self
            .browser
            .new_tab()
            .map_err(|e| eyre!("Failed to create new tab: {e}"))?;

        // Navigate to the page
        tab.navigate_to(url)
            .map_err(|e| eyre!("Failed to navigate to {url}: {e}"))?;

        // Wait for the calendar table to load
        debug!("Waiting for calendar table to load...");
        tab.wait_for_element("table.calendar__table")
            .map_err(|e| eyre!("Calendar table not found (page may have changed): {e}"))?;

        // Get the full HTML content
        let html = tab
            .get_content()
            .map_err(|e| eyre!("Failed to get page content: {e}"))?;

        debug!("Successfully fetched {} bytes of HTML", html.len());

        // Close the tab
        tab.close(true).ok();

        Ok(html)
    }
}

/// Format a date into Forex Factory's week parameter format.
/// e.g., June 1, 2025 -> "jun1.2025"
fn format_week_param(date: NaiveDate) -> String {
    let month = date.format("%b").to_string().to_lowercase();
    let day = date.day();
    let year = date.year();
    format!("{month}{day}.{year}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_week_param() {
        let date = NaiveDate::from_ymd_opt(2025, 6, 4).unwrap();
        assert_eq!(format_week_param(date), "jun4.2025");

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert_eq!(format_week_param(date), "jan15.2025");

        let date = NaiveDate::from_ymd_opt(2025, 12, 25).unwrap();
        assert_eq!(format_week_param(date), "dec25.2025");
    }
}
