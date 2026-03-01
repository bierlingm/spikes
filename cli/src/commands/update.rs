use crate::error::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    println!("Spikes Update Tool");
    println!();
    println!("Current version: {}", current_version);
    println!();
    println!("To update spikes CLI to the latest version:");
    println!();
    println!("  Option 1: Using cargo");
    println!("    cargo install spikes --force");
    println!();
    println!("  Option 2: Using the install script");
    println!("    curl -fsSL https://spikes.sh/install.sh | sh");
    println!();
    println!("  Option 3: Clone and build from source");
    println!("    git clone https://github.com/moritzbierling/spikes.git");
    println!("    cd spikes && cargo build --release --manifest-path cli/Cargo.toml");
    println!();

    // If we're in a git repository, offer to update the widget too
    if Path::new(".git").exists() && Path::new("widget/spikes.js").exists() {
        println!("Local git repository detected with widget/spikes.js");
        println!();
        println!("To update the widget to the latest version:");
        println!("  git pull origin main");
        println!("  cp widget/spikes.js site/spikes.js  (if hosting on spikes.sh)");
        println!();
    }

    // If .spikes directory exists, might be self-hosting
    if Path::new(".spikes").exists() {
        println!("Local spikes project detected.");
        println!();
        println!("To sync all changes:");
        println!("  spikes pull   # Fetch latest from remote");
        println!("  spikes push   # Upload local changes");
        println!();
    }

    println!("For more information:");
    println!("  spikes --version");
    println!("  spikes --help");
    println!("  https://spikes.sh");
    println!();

    Ok(())
}
