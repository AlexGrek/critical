use crate::api;
use crate::context;
use anyhow::Result;
use crate::OutputFormat;
use crit_shared::event_models::Event;

#[derive(serde::Deserialize)]
struct EventsResponse {
    #[allow(dead_code)]
    count: usize,
    events: Vec<Event>,
}

/// Display recent events from the event log.
pub async fn events(output_format: OutputFormat) -> Result<()> {
    let ctx = context::require_current()?;
    let response_json = api::debug_events(&ctx.url, &ctx.token).await?;

    match output_format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&response_json)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&response_json)?),
        OutputFormat::Table => {
            // Deserialize into typed structure for table formatting
            let response: EventsResponse = serde_json::from_value(response_json)?;
            display_events_table(&response.events);
        }
    }

    Ok(())
}

fn display_events_table(events: &[Event]) {
    if events.is_empty() {
        println!("No events found.");
        return;
    }

    // Header
    println!(
        "{:<36} {:<20} {:<15} {:<20} {:<12}",
        "ID", "MOMENT", "PRIORITY", "KIND", "PRINCIPAL"
    );
    println!("{}", "-".repeat(110));

    for event in events {
        let moment_str = event.moment.format("%Y-%m-%d %H:%M:%S").to_string();
        let priority_str = format!("{:?}", event.priority);
        let kind_str = format!("{:?}", event.kind);
        let principal_str = event
            .principal
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());

        // Truncate long IDs
        let id_str = if event.uid.len() > 36 {
            format!("{}...", &event.uid[..33])
        } else {
            event.uid.clone()
        };

        println!(
            "{:<36} {:<20} {:<15} {:<20} {:<12}",
            id_str, moment_str, priority_str, kind_str, principal_str
        );

        // Show details if present
        if let Some(details) = &event.details {
            let details_str = serde_json::to_string(details).unwrap_or_default();
            if !details_str.is_empty() && details_str != "{}" {
                println!("  Details: {}", details_str);
            }
        }

        // Show affected resources if present
        if !event.affects.is_empty() {
            println!("  Affects: {}", event.affects.join(", "));
        }
    }

    println!("\nTotal: {} event(s)", events.len());
}
