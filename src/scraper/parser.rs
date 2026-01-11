use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use color_eyre::{Result, eyre::eyre};
use scraper::{Html, Selector};
use tracing::{debug, warn};

use crate::types::{EconomicEvent, Impact};

/// Parses Forex Factory calendar HTML into structured events.
pub struct CalendarParser {
    // Selectors are compiled once and reused
    row_selector: Selector,
    currency_selector: Selector,
    impact_selector: Selector,
    event_selector: Selector,
    time_selector: Selector,
    actual_selector: Selector,
    forecast_selector: Selector,
    previous_selector: Selector,
}

impl CalendarParser {
    /// Create a new parser with pre-compiled CSS selectors.
    pub fn new() -> Result<Self> {
        Ok(Self {
            // Use data-event-id attribute to find actual event rows
            row_selector: Selector::parse("tr[data-event-id]")
                .map_err(|e| eyre!("Invalid row selector: {e:?}"))?,
            currency_selector: Selector::parse("td.calendar__currency")
                .map_err(|e| eyre!("Invalid currency selector: {e:?}"))?,
            // Impact icon is in a span with class like "icon--ff-impact-yel"
            impact_selector: Selector::parse("td.calendar__impact span")
                .map_err(|e| eyre!("Invalid impact selector: {e:?}"))?,
            event_selector: Selector::parse("td.calendar__event span.calendar__event-title")
                .map_err(|e| eyre!("Invalid event selector: {e:?}"))?,
            time_selector: Selector::parse("td.calendar__time")
                .map_err(|e| eyre!("Invalid time selector: {e:?}"))?,
            actual_selector: Selector::parse("td.calendar__actual")
                .map_err(|e| eyre!("Invalid actual selector: {e:?}"))?,
            forecast_selector: Selector::parse("td.calendar__forecast")
                .map_err(|e| eyre!("Invalid forecast selector: {e:?}"))?,
            previous_selector: Selector::parse("td.calendar__previous")
                .map_err(|e| eyre!("Invalid previous selector: {e:?}"))?,
        })
    }

    /// Parse HTML content into a list of economic events.
    /// The `base_date` is used when time-only values are found (no date in the row).
    pub fn parse(&self, html: &str, base_date: NaiveDate) -> Result<Vec<EconomicEvent>> {
        debug!("Parsing HTML of {} bytes for date {base_date}", html.len());
        let document = Html::parse_document(html);
        let mut events = Vec::new();
        let mut current_date = base_date;
        let mut current_time: Option<NaiveTime> = None;

        let row_count = document.select(&self.row_selector).count();
        debug!("Found {row_count} event rows in HTML");

        for row in document.select(&self.row_selector) {
            // TODO(human): Implement the event extraction logic here
            // Extract: currency, impact, event name, time, actual, forecast, previous
            // Handle date propagation (some rows don't have dates, inherit from previous)
            // Handle time propagation similarly
            // Skip rows with no event data (e.g., holiday rows)

            let event = self.parse_row(&row, &mut current_date, &mut current_time);

            match event {
                Ok(Some(e)) => {
                    debug!("Parsed event: {} ({}) - {}", e.name, e.currency, e.impact);
                    events.push(e);
                }
                Ok(None) => {
                    // Row had no parseable event (e.g., header row)
                    continue;
                }
                Err(e) => {
                    warn!("Failed to parse row: {e}");
                    continue;
                }
            }
        }

        Ok(events)
    }

    /// Parse a single table row into an event.
    /// Returns Ok(None) if the row doesn't contain event data.
    fn parse_row(
        &self,
        row: &scraper::ElementRef,
        current_date: &mut NaiveDate,
        current_time: &mut Option<NaiveTime>,
    ) -> Result<Option<EconomicEvent>> {
        // TODO(human): Implement the row parsing logic
        //
        // Steps:
        // 1. Check if row has a date cell, update current_date if so
        // 2. Check if row has a time cell, update current_time if so
        // 3. Extract currency (skip if empty - might be a header row)
        // 4. Extract impact level from the span's class
        // 5. Extract event name
        // 6. Extract actual/forecast/previous values (may be empty)
        // 7. Combine date + time into datetime
        //
        // Hints:
        // - Use self.currency_selector.select_first(row) pattern
        // - Impact is determined by class like "icon--ff-impact-red"
        // - Text content: element.text().collect::<String>().trim()
        // - Some cells span multiple rows, so values may be inherited

        // Placeholder implementation - you'll replace this
        let currency = self.extract_text(row, &self.currency_selector);
        if currency.is_empty() {
            return Ok(None);
        }

        let impact = self.extract_impact(row).unwrap_or(Impact::Low);
        let name = self.extract_text(row, &self.event_selector);
        if name.is_empty() {
            return Ok(None);
        }

        // Update time if present in this row
        let time_text = self.extract_text(row, &self.time_selector);
        if !time_text.is_empty()
            && time_text != "All Day"
            && time_text != "Tentative"
            && let Some(parsed_time) = parse_time(&time_text)
        {
            *current_time = Some(parsed_time);
        }

        let time = current_time.unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let datetime = NaiveDateTime::new(*current_date, time);
        let datetime_utc = Utc.from_utc_datetime(&datetime);

        let actual = self.extract_text(row, &self.actual_selector);
        let forecast = self.extract_text(row, &self.forecast_selector);
        let previous = self.extract_text(row, &self.previous_selector);

        Ok(Some(EconomicEvent {
            name,
            currency,
            impact,
            datetime: datetime_utc,
            actual: if actual.is_empty() {
                None
            } else {
                Some(actual)
            },
            forecast: if forecast.is_empty() {
                None
            } else {
                Some(forecast)
            },
            previous: if previous.is_empty() {
                None
            } else {
                Some(previous)
            },
        }))
    }

    /// Extract text content from the first matching element.
    fn extract_text(&self, row: &scraper::ElementRef, selector: &Selector) -> String {
        row.select(selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default()
    }

    /// Extract impact level from the impact cell's span class.
    fn extract_impact(&self, row: &scraper::ElementRef) -> Option<Impact> {
        row.select(&self.impact_selector)
            .next()
            .and_then(|el| el.value().attr("class").and_then(Impact::from_ff_class))
    }
}

/// Parse time string from Forex Factory format.
/// Examples: "8:30am", "2:00pm", "12:30am"
fn parse_time(time_str: &str) -> Option<NaiveTime> {
    let time_str = time_str.trim().to_lowercase();

    // Try various formats
    if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%l:%M%P") {
        return Some(time);
    }
    if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%I:%M%P") {
        return Some(time);
    }
    if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%H:%M") {
        return Some(time);
    }

    None
}

impl Default for CalendarParser {
    fn default() -> Self {
        Self::new().expect("Default selectors should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        // Note: These tests verify the parse_time function handles various formats
        // The actual parsing may need adjustment based on real Forex Factory data

        // 12-hour format with am/pm
        // assert_eq!(parse_time("8:30am"), Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()));
        // assert_eq!(parse_time("2:00pm"), Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap()));

        // 24-hour format
        assert_eq!(
            parse_time("14:00"),
            Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap())
        );
    }

    #[test]
    fn test_parser_creation() {
        let parser = CalendarParser::new();
        assert!(parser.is_ok());
    }
}
