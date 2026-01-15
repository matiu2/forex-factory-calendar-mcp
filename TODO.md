# Forex Factory Calendar MCP Server - TODO

## Overview
MCP server to query economic events from Forex Factory calendar.
- Data source: Web scraping via HTTP requests (async reqwest)
- Currency pairs: Query returns events for both currencies (OR logic)
- Timezone: Events are returned in the user's local timezone

## Completed
- [x] Define core types: EconomicEvent, Impact (Low/Medium/High), EventQuery
- [x] Set up HTTP scraper (async reqwest with browser-like headers)
- [x] Implement HTML parsing for calendar events
- [x] Set up MCP server with rmcp
- [x] Implement tools:
  - `query_events` - Filter by currency, date range, impact level
  - `get_week_around` - Get events around a specific date
  - `get_today_events` - Get today's events
  - `get_week_events` - Get this week's events
- [x] Test end-to-end with actual Forex Factory data
- [x] Add local timezone support

## Completed: Impact Display and Country Name Support

### 1. Add Country-to-Currency Mapping
- [x] Create `src/types/currency.rs` module with country name → currency code mapping
- [x] Support common variations: "Canada", "Canadian", "CAD" all → "CAD"
- [x] Handle case-insensitivity

### 2. Update Currency Parsing
- [x] Modify `parse_currencies()` in `src/mcp/tools.rs` to resolve country names
- [x] Keep backwards compatibility with existing currency codes

### 3. Add Stars to EventResult
- [x] Add `impact_stars: u8` field to `EventResult` struct
- [x] Update `From<EconomicEvent>` impl to populate stars

### 4. Testing & Finalization
- [x] Add tests for country name resolution
- [x] Add tests for stars in EventResult
- [x] cargo clippy && cargo fmt
- [x] Commit and push

## Optional / Future
- [ ] Add caching layer to minimize HTTP requests
- [ ] Add more sophisticated date parsing (e.g., "next week", "last month")
- [ ] Support for commodity events (Oil, Gold)
- [ ] Parse date headers from calendar rows to handle multi-day views

## Usage

Build:
```bash
cargo build --release
```

Install to cargo bin:
```bash
cargo install --path .
```

Configure in Claude Code:
```bash
claude mcp add forex-calendar --transport stdio -- ~/.cargo/bin/forex-factory-calendar-mcp
```

Or configure in Claude Desktop (`~/.config/claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "forex-calendar": {
      "command": "~/.cargo/bin/forex-factory-calendar-mcp"
    }
  }
}
```

Example queries:
- "What high-impact USD events are happening this week?"
- "Show me AUD/CHF events around June 4th 2025"
- "Get all economic events for today"
- "Find 2 star or above events for EUR this week"
