# Forex Factory Calendar MCP Server - TODO

## Overview
MCP server to query economic events from Forex Factory calendar.
- Data source: Web scraping with headless browser (Chromium)
- Currency pairs: Query returns events for both currencies (OR logic)

## Completed
- [x] Define core types: EconomicEvent, Impact (Low/Medium/High), EventQuery
- [x] Set up headless browser (headless_chrome crate)
- [x] Implement HTML parsing for calendar events
- [x] Set up MCP server with rmcp
- [x] Implement tools:
  - `query_events` - Filter by currency, date range, impact level
  - `get_week_around` - Get events around a specific date
  - `get_today_events` - Get today's events
  - `get_week_events` - Get this week's events

## In Progress
- [ ] Test end-to-end with actual Forex Factory data

## Optional / Future
- [ ] Add caching layer to minimize browser requests
- [ ] Add more sophisticated date parsing (e.g., "next week", "last month")
- [ ] Support for commodity events (Oil, Gold)

## Usage

Build:
```bash
cargo build --release
```

Configure in Claude Desktop (`~/.config/claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "forex-calendar": {
      "command": "/path/to/forex-factory-calendar-mcp"
    }
  }
}
```

Example queries:
- "What high-impact USD events are happening this week?"
- "Show me AUD/CHF events around June 4th 2025"
- "Get all economic events for today"
