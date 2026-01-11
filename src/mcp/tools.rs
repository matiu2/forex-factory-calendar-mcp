use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for querying economic events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryEventsParams {
    /// Currency pair or single currency (e.g., "AUD/CHF", "USD", "EUR,GBP")
    /// For pairs, events for both currencies will be returned.
    #[serde(default)]
    pub currencies: Option<String>,

    /// Start date in YYYY-MM-DD format
    #[serde(default)]
    pub from_date: Option<String>,

    /// End date in YYYY-MM-DD format
    #[serde(default)]
    pub to_date: Option<String>,

    /// Minimum impact level: "low", "medium", "high" or 1-3 stars
    #[serde(default)]
    pub min_impact: Option<String>,
}

/// Parameters for getting events around a specific date
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WeekAroundParams {
    /// Center date in YYYY-MM-DD format
    pub date: String,

    /// Currency pair or single currency (optional)
    #[serde(default)]
    pub currencies: Option<String>,

    /// Minimum impact level (optional)
    #[serde(default)]
    pub min_impact: Option<String>,
}

/// Result event returned by the MCP tools
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventResult {
    /// Event name
    pub name: String,

    /// Currency code (e.g., "USD", "EUR")
    pub currency: String,

    /// Impact level: "low", "medium", or "high"
    pub impact: String,

    /// Event date and time in ISO 8601 format
    pub datetime: String,

    /// Actual value (if released)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,

    /// Forecasted value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forecast: Option<String>,

    /// Previous period's value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,
}

impl QueryEventsParams {
    /// Parse currencies from the string (handles "AUD/CHF", "EUR,GBP", "USD")
    pub fn parse_currencies(&self) -> Vec<String> {
        self.currencies
            .as_ref()
            .map(|s| {
                s.split(['/', ',', '-', ' '])
                    .map(|c| c.trim().to_uppercase())
                    .filter(|c| !c.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse from_date string to NaiveDate
    pub fn parse_from_date(&self) -> Option<NaiveDate> {
        self.from_date
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// Parse to_date string to NaiveDate
    pub fn parse_to_date(&self) -> Option<NaiveDate> {
        self.to_date
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// Parse min_impact string to impact level (1-3 or "low"/"medium"/"high")
    pub fn parse_min_impact(&self) -> Option<crate::types::Impact> {
        use crate::types::Impact;

        self.min_impact.as_ref().and_then(|s| {
            let s = s.trim().to_lowercase();
            match s.as_str() {
                "low" | "1" => Some(Impact::Low),
                "medium" | "med" | "2" => Some(Impact::Medium),
                "high" | "3" => Some(Impact::High),
                _ => None,
            }
        })
    }
}

impl From<crate::types::EconomicEvent> for EventResult {
    fn from(event: crate::types::EconomicEvent) -> Self {
        Self {
            name: event.name,
            currency: event.currency,
            impact: event.impact.to_string().to_lowercase(),
            datetime: event.datetime.to_rfc3339(),
            actual: event.actual,
            forecast: event.forecast,
            previous: event.previous,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_currencies() {
        let params = QueryEventsParams {
            currencies: Some("AUD/CHF".to_string()),
            from_date: None,
            to_date: None,
            min_impact: None,
        };
        assert_eq!(params.parse_currencies(), vec!["AUD", "CHF"]);

        let params = QueryEventsParams {
            currencies: Some("EUR,GBP,USD".to_string()),
            from_date: None,
            to_date: None,
            min_impact: None,
        };
        assert_eq!(params.parse_currencies(), vec!["EUR", "GBP", "USD"]);

        let params = QueryEventsParams {
            currencies: None,
            from_date: None,
            to_date: None,
            min_impact: None,
        };
        assert!(params.parse_currencies().is_empty());
    }

    #[test]
    fn test_parse_dates() {
        let params = QueryEventsParams {
            currencies: None,
            from_date: Some("2025-06-04".to_string()),
            to_date: Some("2025-06-10".to_string()),
            min_impact: None,
        };

        assert_eq!(
            params.parse_from_date(),
            Some(NaiveDate::from_ymd_opt(2025, 6, 4).unwrap())
        );
        assert_eq!(
            params.parse_to_date(),
            Some(NaiveDate::from_ymd_opt(2025, 6, 10).unwrap())
        );
    }

    #[test]
    fn test_parse_min_impact() {
        use crate::types::Impact;

        let params = QueryEventsParams {
            currencies: None,
            from_date: None,
            to_date: None,
            min_impact: Some("high".to_string()),
        };
        assert_eq!(params.parse_min_impact(), Some(Impact::High));

        let params = QueryEventsParams {
            currencies: None,
            from_date: None,
            to_date: None,
            min_impact: Some("2".to_string()),
        };
        assert_eq!(params.parse_min_impact(), Some(Impact::Medium));
    }
}
