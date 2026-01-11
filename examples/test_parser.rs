//! Test the parser with actual HTML from Forex Factory
use chrono::{Local, NaiveDate};
use color_eyre::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use scraper::{Html, Selector};
use std::time::Duration;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("Building HTTP client...\n");

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        ),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

    let client = Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("Fetching this week's calendar...\n");

    let html = client
        .get("https://www.forexfactory.com/calendar?week=this")
        .send()?
        .text()?;

    println!("Got {} bytes of HTML\n", html.len());

    // Use the parser's selectors to test parsing
    let document = Html::parse_document(&html);
    let row_selector = Selector::parse("tr[data-event-id]").unwrap();
    let currency_selector = Selector::parse("td.calendar__currency").unwrap();
    let event_selector = Selector::parse("td.calendar__event span.calendar__event-title").unwrap();
    let time_selector = Selector::parse("td.calendar__time").unwrap();
    let impact_selector = Selector::parse("td.calendar__impact span").unwrap();

    let mut event_count = 0;
    for row in document.select(&row_selector) {
        // Extract currency
        let currency: String = row
            .select(&currency_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if currency.is_empty() {
            continue;
        }

        // Extract event name
        let event_name: String = row
            .select(&event_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if event_name.is_empty() {
            continue;
        }

        // Extract time
        let time: String = row
            .select(&time_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Extract impact
        let impact: String = row
            .select(&impact_selector)
            .next()
            .and_then(|el| el.value().attr("class"))
            .map(|s| {
                if s.contains("red") {
                    "High"
                } else if s.contains("ora") {
                    "Medium"
                } else if s.contains("yel") {
                    "Low"
                } else {
                    "None"
                }
            })
            .unwrap_or("Unknown")
            .to_string();

        println!(
            "[{}] {} - {} ({}) @ {}",
            event_count + 1,
            currency,
            event_name,
            impact,
            time
        );
        event_count += 1;

        if event_count >= 20 {
            println!("\n... and more events (showing first 20)");
            break;
        }
    }

    println!("\nTotal events found: {}", event_count);

    Ok(())
}
