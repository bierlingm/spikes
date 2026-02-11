use crate::error::Result;
use crate::output::print_json;
use crate::storage;
use crate::tui;

pub fn run(json: bool) -> Result<()> {
    let spikes = storage::load_spikes()?;

    if json {
        print_json(&spikes);
        return Ok(());
    }

    tui::run(spikes)?;
    Ok(())
}
