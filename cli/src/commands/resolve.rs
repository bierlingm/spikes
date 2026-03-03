use crate::error::Result;
use crate::output::print_json;
use crate::storage::{find_spike_by_id, load_spikes, save_spikes};
use crate::spike::Spike;

pub struct ResolveOptions {
    pub id: String,
    pub unresolve: bool,
    pub json: bool,
}

/// Get current time as ISO 8601 timestamp
fn current_timestamp() -> String {
    chrono::Local::now().to_rfc3339()
}

pub fn run(options: ResolveOptions) -> Result<()> {
    let mut spikes = load_spikes()?;
    let spike = find_spike_by_id(&spikes, &options.id)?;
    
    // Find and update the spike
    let mut updated_spike: Option<Spike> = None;
    for s in &mut spikes {
        if s.id == spike.id {
            if options.unresolve {
                s.resolved = None;
                s.resolved_at = None;
            } else {
                s.resolved = Some(true);
                s.resolved_at = Some(current_timestamp());
            }
            updated_spike = Some(s.clone());
            break;
        }
    }
    
    save_spikes(&spikes)?;
    
    let updated = updated_spike.expect("spike should exist");
    
    if options.json {
        print_json(&updated);
    } else {
        if options.unresolve {
            println!("Unresolved spike {}.", updated.id);
        } else {
            println!("Resolved spike {}.", updated.id);
        }
    }
    
    Ok(())
}
