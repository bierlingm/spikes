mod commands;
mod error;
mod output;
mod spike;
mod storage;
mod tui;

use clap::{Parser, Subcommand};
use commands::deploy::DeployOptions;
use commands::export::ExportFormat;
use commands::inject::InjectOptions;
use commands::list::ListOptions;
use commands::pull::PullOptions;
use commands::push::PushOptions;
use commands::serve::ServeOptions;

#[derive(Parser)]
#[command(name = "spikes")]
#[command(about = "Feedback collection for static mockups", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a .spikes/ directory
    Init {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List all spikes
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Filter by page name
        #[arg(long)]
        page: Option<String>,

        /// Filter by reviewer name
        #[arg(long)]
        reviewer: Option<String>,

        /// Filter by rating (love, like, meh, no)
        #[arg(long)]
        rating: Option<String>,
    },

    /// Show a single spike by ID
    Show {
        /// Spike ID (or prefix)
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export all spikes
    Export {
        /// Output format: json, csv, or jsonl
        #[arg(long, short, default_value = "json")]
        format: String,
    },

    /// Show elements with most feedback
    Hotspots {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List all reviewers who left feedback
    Reviewers {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Add widget script tag to HTML files
    Inject {
        /// Directory containing HTML files
        directory: String,

        /// Remove widget script tags instead of adding
        #[arg(long)]
        remove: bool,

        /// URL for widget script (default: /spikes.js for local serve)
        #[arg(long)]
        widget_url: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Start local development server
    Serve {
        /// Port to listen on (default: 3847)
        #[arg(long, short, default_value = "3847")]
        port: u16,

        /// Directory to serve (default: current directory)
        #[arg(long, short, default_value = ".")]
        dir: String,
    },

    /// Deploy backend to Cloudflare
    Deploy {
        #[command(subcommand)]
        backend: DeployBackend,
    },

    /// Fetch spikes from remote and merge with local
    Pull {
        /// Remote endpoint URL (or from .spikes/config.toml)
        #[arg(long)]
        endpoint: Option<String>,

        /// Auth token (or from .spikes/config.toml)
        #[arg(long)]
        token: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Upload local spikes to remote
    Push {
        /// Remote endpoint URL (or from .spikes/config.toml)
        #[arg(long)]
        endpoint: Option<String>,

        /// Auth token (or from .spikes/config.toml)
        #[arg(long)]
        token: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show version
    Version,

    /// Interactive TUI dashboard
    Dashboard {
        /// Output as JSON (non-interactive)
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum DeployBackend {
    /// Scaffold Cloudflare Worker + D1 for multi-reviewer sync
    Cloudflare {
        /// Output directory (default: ./spikes-worker)
        #[arg(long)]
        dir: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { json } => commands::init::run(json),
        Commands::List {
            json,
            page,
            reviewer,
            rating,
        } => commands::list::run(ListOptions {
            json,
            page,
            reviewer,
            rating,
        }),
        Commands::Show { id, json } => commands::show::run(&id, json),
        Commands::Export { format } => {
            let fmt = match format.parse::<ExportFormat>() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };
            commands::export::run(fmt)
        }
        Commands::Hotspots { json } => commands::hotspots::run(json),
        Commands::Reviewers { json } => commands::reviewers::run(json),
        Commands::Inject {
            directory,
            remove,
            widget_url,
            json,
        } => commands::inject::run(InjectOptions {
            directory,
            remove,
            widget_url,
            json,
        }),
        Commands::Serve { port, dir } => commands::serve::run(ServeOptions {
            port,
            directory: dir,
        }),
        Commands::Deploy { backend } => match backend {
            DeployBackend::Cloudflare { dir, json } => {
                commands::deploy::run(DeployOptions { dir, json })
            }
        },
        Commands::Pull {
            endpoint,
            token,
            json,
        } => commands::pull::run(PullOptions {
            endpoint,
            token,
            json,
        }),
        Commands::Push {
            endpoint,
            token,
            json,
        } => commands::push::run(PushOptions {
            endpoint,
            token,
            json,
        }),
        Commands::Version => {
            println!("spikes {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Dashboard { json } => commands::dashboard::run(json),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
