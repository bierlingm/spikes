use std::io::{self, BufRead, Write};

use crate::error::Result;
use crate::output::print_json;
use crate::storage::{find_spike_by_id, load_spikes, save_spikes};

pub struct DeleteOptions {
    pub id: String,
    pub force: bool,
    pub json: bool,
}

pub fn run(options: DeleteOptions) -> Result<()> {
    let mut spikes = load_spikes()?;
    let spike = find_spike_by_id(&spikes, &options.id)?;
    
    // If not --force, prompt for confirmation
    if !options.force {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        loop {
            print!(
                "Delete spike '{}' on page '{}'? [y/N] ",
                &spike.id[..8.min(spike.id.len())],
                spike.page
            );
            stdout.flush()?;
            
            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;
            
            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => break,
                "n" | "no" | "" => {
                    if options.json {
                        print_json(&serde_json::json!({
                            "deleted": false,
                            "id": spike.id,
                            "message": "Cancelled"
                        }));
                    } else {
                        println!("Cancelled.");
                    }
                    return Ok(());
                }
                _ => continue,
            }
        }
    }
    
    // Remove the spike
    spikes.retain(|s| s.id != spike.id);
    save_spikes(&spikes)?;
    
    if options.json {
        print_json(&serde_json::json!({
            "deleted": true,
            "id": spike.id
        }));
    } else {
        println!("Deleted spike {}.", spike.id);
    }
    
    Ok(())
}
