use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_router,
};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::service::CalendarService;
use crate::types::{EventQuery, Impact};

use super::tools::{EventResult, QueryEventsParams, WeekAroundParams};

/// MCP Server for Forex Factory Calendar
#[derive(Clone)]
pub struct ForexCalendarServer {
    service: Arc<RwLock<Option<CalendarService>>>,
    tool_router: ToolRouter<Self>,
}

impl ForexCalendarServer {
    /// Create a new server instance.
    /// The calendar service is lazily initialized on first use.
    pub fn new() -> Self {
        Self {
            service: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    /// Get or initialize the calendar service
    async fn get_service(&self) -> Result<(), McpError> {
        let mut service_guard = self.service.write().await;
        if service_guard.is_none() {
            info!("Initializing calendar service...");
            match CalendarService::new() {
                Ok(svc) => {
                    *service_guard = Some(svc);
                    info!("Calendar service initialized successfully");
                }
                Err(e) => {
                    error!("Failed to initialize calendar service: {e}");
                    return Err(McpError::internal_error(
                        format!("Failed to initialize browser: {e}"),
                        None,
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Default for ForexCalendarServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl ForexCalendarServer {
    /// Query economic events from Forex Factory calendar.
    #[tool(
        description = "Query economic events from Forex Factory calendar. Supports filtering by currency (e.g., 'USD', 'AUD/CHF'), date range (YYYY-MM-DD format), and minimum impact level ('low', 'medium', 'high' or 1-3)."
    )]
    async fn query_events(
        &self,
        Parameters(params): Parameters<QueryEventsParams>,
    ) -> Result<String, McpError> {
        self.get_service().await?;

        let service_guard = self.service.read().await;
        let service = service_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Calendar service not initialized", None))?;

        // Build query from params
        let mut query = EventQuery::new();

        let currencies = params.parse_currencies();
        if !currencies.is_empty() {
            query = query.with_currencies(currencies);
        }

        if let Some(from) = params.parse_from_date() {
            if let Some(to) = params.parse_to_date() {
                query = query.with_date_range(from, to);
            } else {
                query = query.with_date_range(from, from);
            }
        }

        if let Some(impact) = params.parse_min_impact() {
            query = query.with_min_impact(impact);
        }

        // Execute query
        match service.query_events(&query).await {
            Ok(events) => {
                let results: Vec<EventResult> = events.into_iter().map(Into::into).collect();
                serde_json::to_string_pretty(&results).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize results: {e}"), None)
                })
            }
            Err(e) => {
                error!("Query failed: {e}");
                Err(McpError::internal_error(format!("Query failed: {e}"), None))
            }
        }
    }

    /// Get events for the week around a specific date.
    #[tool(
        description = "Get economic events for the week around a specific date. Returns events 3 days before and after the specified date."
    )]
    async fn get_week_around(
        &self,
        Parameters(params): Parameters<WeekAroundParams>,
    ) -> Result<String, McpError> {
        self.get_service().await?;

        let service_guard = self.service.read().await;
        let service = service_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Calendar service not initialized", None))?;

        // Parse the center date
        let date = chrono::NaiveDate::parse_from_str(&params.date, "%Y-%m-%d").map_err(|e| {
            McpError::invalid_params(format!("Invalid date format. Use YYYY-MM-DD: {e}"), None)
        })?;

        // Build query
        let mut query = EventQuery::new().with_week_around(date);

        if let Some(ref currencies) = params.currencies {
            let parsed: Vec<String> = currencies
                .split(['/', ',', '-', ' '])
                .map(|c| c.trim().to_uppercase())
                .filter(|c| !c.is_empty())
                .collect();
            if !parsed.is_empty() {
                query = query.with_currencies(parsed);
            }
        }

        if let Some(ref min_impact) = params.min_impact {
            let impact = match min_impact.trim().to_lowercase().as_str() {
                "low" | "1" => Some(Impact::Low),
                "medium" | "med" | "2" => Some(Impact::Medium),
                "high" | "3" => Some(Impact::High),
                _ => None,
            };
            if let Some(imp) = impact {
                query = query.with_min_impact(imp);
            }
        }

        // Fetch and filter events
        match service.get_week_events_for(date).await {
            Ok(events) => {
                let min_impact = query.min_impact.unwrap_or(Impact::Low);
                let filtered: Vec<EventResult> = events
                    .into_iter()
                    .filter(|e| e.meets_impact(min_impact))
                    .filter(|e| e.matches_currencies(&query.currencies))
                    .filter(|e| query.datetime_in_range(&e.datetime))
                    .map(Into::into)
                    .collect();

                serde_json::to_string_pretty(&filtered).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize results: {e}"), None)
                })
            }
            Err(e) => {
                error!("Failed to get week events: {e}");
                Err(McpError::internal_error(
                    format!("Failed to get week events: {e}"),
                    None,
                ))
            }
        }
    }

    /// Get today's economic events.
    #[tool(description = "Get all economic events scheduled for today.")]
    async fn get_today_events(&self) -> Result<String, McpError> {
        self.get_service().await?;

        let service_guard = self.service.read().await;
        let service = service_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Calendar service not initialized", None))?;

        match service.get_today_events().await {
            Ok(events) => {
                let results: Vec<EventResult> = events.into_iter().map(Into::into).collect();
                serde_json::to_string_pretty(&results).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize results: {e}"), None)
                })
            }
            Err(e) => {
                error!("Failed to get today's events: {e}");
                Err(McpError::internal_error(
                    format!("Failed to get today's events: {e}"),
                    None,
                ))
            }
        }
    }

    /// Get this week's economic events.
    #[tool(description = "Get all economic events scheduled for the current week.")]
    async fn get_week_events(&self) -> Result<String, McpError> {
        self.get_service().await?;

        let service_guard = self.service.read().await;
        let service = service_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Calendar service not initialized", None))?;

        match service.get_week_events().await {
            Ok(events) => {
                let results: Vec<EventResult> = events.into_iter().map(Into::into).collect();
                serde_json::to_string_pretty(&results).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize results: {e}"), None)
                })
            }
            Err(e) => {
                error!("Failed to get week events: {e}");
                Err(McpError::internal_error(
                    format!("Failed to get week events: {e}"),
                    None,
                ))
            }
        }
    }
}

impl ServerHandler for ForexCalendarServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "forex-factory-calendar".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Query economic events from Forex Factory calendar. \
                 Use query_events for filtered queries, get_week_around for date-centered queries, \
                 or get_today_events/get_week_events for quick access to current events."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
