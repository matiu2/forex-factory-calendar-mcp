# Forex Factory Calendar MCP Server - TODO

## Overview
MCP server to query economic events from Forex Factory calendar.
- Data source: Web scraping with headless browser
- Currency pairs: Query returns events for both currencies (OR logic)

## Phase 1: Core Types and Data Model
- [ ] Define `EconomicEvent` struct (name, currency, impact, datetime, actual, forecast, previous)
- [ ] Define `Impact` enum (Low, Medium, High) with star conversion
- [ ] Define `Currency` enum or use String with validation
- [ ] Define query parameters struct (currencies, date range, min_impact)

## Phase 2: Web Scraping
- [ ] Set up headless browser (chromiumoxide or fantoccini)
- [ ] Implement calendar page fetching with proper User-Agent
- [ ] Parse HTML to extract events
- [ ] Handle pagination for date ranges
- [ ] Add caching layer to minimize requests

## Phase 3: MCP Server
- [ ] Set up rmcp server structure
- [ ] Implement `query_events` tool
- [ ] Implement `get_week_events` tool
- [ ] Implement `get_today_events` tool
- [ ] Add proper error handling and responses

## Phase 4: Polish
- [ ] Add configuration (cache TTL, browser path)
- [ ] Write tests
- [ ] Documentation
- [ ] cargo clippy / cargo fmt

## Current Status
Starting Phase 1...
