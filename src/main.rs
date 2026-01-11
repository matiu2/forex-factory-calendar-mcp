mod mcp;
mod scraper;
mod service;
mod types;

use color_eyre::Result;
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use tracing::info;

use mcp::ForexCalendarServer;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Initialize tracing - write to stderr to keep stdout clean for MCP
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Forex Factory Calendar MCP Server starting...");

    // Create the server
    let server = ForexCalendarServer::new();

    // Create stdio transport and serve
    let transport = (stdin(), stdout());
    let service = server.serve(transport).await?;

    info!("Server initialized, waiting for requests...");

    // Wait for the service to complete
    service.waiting().await?;

    info!("Server shutting down");
    Ok(())
}
