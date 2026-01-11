# Forex Factory Calendar MCP Server

An MCP (Model Context Protocol) server that queries economic events from the [Forex Factory](https://www.forexfactory.com/) calendar. Use it with Claude Code, Claude Desktop, or any MCP-compatible client to get real-time forex economic calendar data.

## Features

- Query economic events by currency, date range, and impact level
- Support for currency pairs (e.g., "AUD/CHF" returns events for both currencies)
- Filter by impact level (low/medium/high or 1-3 stars)
- **All event times are returned in your local timezone**

## Installation

### Prerequisites

- Rust 2024 edition (1.85+)

### Build from source

```bash
git clone https://github.com/matiu2/forex-factory-calendar-mcp.git
cd forex-factory-calendar-mcp
cargo build --release
```

### Install to cargo bin

```bash
cargo install --path .
```

## Configuration

### Claude Code

```bash
claude mcp add forex-news --transport stdio -- ~/.cargo/bin/forex-factory-calendar-mcp
```

### Claude Desktop

Add to your config file (`~/.config/claude/claude_desktop_config.json` on Linux, `~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "forex-news": {
      "command": "~/.cargo/bin/forex-factory-calendar-mcp"
    }
  }
}
```

## Available Tools

### `get_week_events`

Get all economic events for the current week.

**Example prompt:** "What economic events are happening this week?"

### `get_today_events`

Get all economic events scheduled for today.

**Example prompt:** "Show me today's forex events"

### `query_events`

Query events with filters.

| Parameter | Description | Example |
|-----------|-------------|---------|
| `currencies` | Currency pair or list | `"AUD/CHF"`, `"USD"`, `"EUR,GBP"` |
| `from_date` | Start date (YYYY-MM-DD) | `"2025-06-01"` |
| `to_date` | End date (YYYY-MM-DD) | `"2025-06-07"` |
| `min_impact` | Minimum impact level | `"high"`, `"medium"`, `"2"` |

**Example prompts:**
- "Find high-impact USD events this week"
- "Show me AUD and NZD events for the next 3 days"
- "What 2-star or above EUR events are coming up?"

### `get_week_around`

Get events for the week around a specific date (3 days before and after).

| Parameter | Description | Example |
|-----------|-------------|---------|
| `date` | Center date (YYYY-MM-DD) | `"2025-06-04"` |
| `currencies` | Optional currency filter | `"AUD/CHF"` |
| `min_impact` | Optional impact filter | `"high"` |

**Example prompt:** "Show me AUD/CHF events around June 4th 2025"

## Output Format

Events are returned as JSON with the following fields:

```json
{
  "name": "Core CPI m/m",
  "currency": "USD",
  "impact": "high",
  "datetime": "2026-01-15T00:30:00+10:00",
  "forecast": "0.3%",
  "previous": "0.2%"
}
```

- `datetime` is in **your local timezone** (note the timezone offset like `+10:00`)
- `impact` is one of: `low`, `medium`, `high`
- `actual`, `forecast`, and `previous` are included when available

## Timezone Handling

All event times are converted to your system's local timezone. The datetime field includes the timezone offset (e.g., `+10:00` for AEST, `-05:00` for EST), so you always know exactly when events occur in your local time.

## License

MIT
