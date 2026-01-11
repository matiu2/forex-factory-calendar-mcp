//! Debug script to examine what HTML structure Forex Factory returns
use color_eyre::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use scraper::{Html, Selector};
use std::time::Duration;

fn main() -> Result<()> {
    color_eyre::install()?;

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

    let document = Html::parse_document(&html);

    // Look at rows with data-event-id (actual event rows)
    let event_row_selector = Selector::parse("tr[data-event-id]").unwrap();
    let event_rows: Vec<_> = document.select(&event_row_selector).collect();
    println!("Found {} rows with data-event-id\n", event_rows.len());

    // Print first 3 event rows in detail
    for (i, row) in event_rows.iter().take(3).enumerate() {
        println!("--- Event Row {} ---", i + 1);
        println!("Classes: {:?}", row.value().attr("class"));
        println!("data-event-id: {:?}", row.value().attr("data-event-id"));

        // Try to extract currency
        let currency_sel = Selector::parse("td.calendar__currency").unwrap();
        if let Some(cell) = row.select(&currency_sel).next() {
            let text: String = cell.text().collect();
            println!("Currency: '{}'", text.trim());
        }

        // Try to extract time
        let time_sel = Selector::parse("td.calendar__time").unwrap();
        if let Some(cell) = row.select(&time_sel).next() {
            let text: String = cell.text().collect();
            println!("Time: '{}'", text.trim());
        }

        // Try to extract event name
        let event_sel = Selector::parse("td.calendar__event").unwrap();
        if let Some(cell) = row.select(&event_sel).next() {
            let text: String = cell.text().collect();
            println!("Event: '{}'", text.trim());
        }

        // Try to extract impact
        let impact_sel = Selector::parse("td.calendar__impact span").unwrap();
        if let Some(span) = row.select(&impact_sel).next() {
            println!("Impact class: {:?}", span.value().attr("class"));
            println!("Impact title: {:?}", span.value().attr("title"));
        }

        println!();
    }

    // Print a snippet of an event row HTML
    if let Some(row) = event_rows.first() {
        println!("First event row HTML:\n{}\n", row.html());
    }

    Ok(())
}
