mod commands;
mod config;
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
use commands::login::LoginOptions;
use commands::pull::PullOptions;
use commands::push::PushOptions;
use commands::serve::ServeOptions;
use commands::share::ShareOptions;
use commands::shares::SharesOptions;
use commands::unshare::UnshareOptions;

#[derive(Parser)]
#[command(name = "spikes")]
#[command(about = "Feedback collection for static mockups", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Port for dev server (magic mode)
    #[arg(long, short, default_value = "3847", global = true)]
    port: u16,
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

        /// Enable review mode with spike markers on pages
        #[arg(long, short)]
        marked: bool,
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

        /// Pull from a public share URL (e.g., https://spikes.sh/s/project-slug)
        #[arg(long)]
        from: Option<String>,

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

    /// Sync with remote (pull then push)
    Sync {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage remote endpoint configuration
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    /// Show current configuration
    Config {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show version
    Version,

    /// Log in to spikes.sh hosted service
    Login {
        /// Auth token (or enter interactively)
        #[arg(long)]
        token: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Interactive TUI dashboard
    Dashboard {
        /// Output as JSON (non-interactive)
        #[arg(long)]
        json: bool,
    },

    /// Upload a directory to spikes.sh for instant sharing
    Share {
        /// Directory to upload
        directory: String,

        /// Custom name for the share URL
        #[arg(long)]
        name: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List your shared projects on spikes.sh
    Shares {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Delete a shared project from spikes.sh
    Unshare {
        /// Share slug to delete
        slug: String,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,

        /// Output as JSON
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

#[derive(Subcommand)]
enum RemoteAction {
    /// Add or update remote endpoint
    Add {
        /// Endpoint URL
        endpoint: String,

        /// Auth token
        #[arg(long)]
        token: Option<String>,

        /// Use spikes.sh hosted backend
        #[arg(long)]
        hosted: bool,
    },

    /// Remove remote configuration
    Remove,

    /// Show current remote configuration
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        // Magic mode: no subcommand = auto-serve current directory
        None => commands::magic::run(cli.port),
        Some(Commands::Init { json }) => commands::init::run(json),
        Some(Commands::List {
            json,
            page,
            reviewer,
            rating,
        }) => commands::list::run(ListOptions {
            json,
            page,
            reviewer,
            rating,
        }),
        Some(Commands::Show { id, json }) => commands::show::run(&id, json),
        Some(Commands::Export { format }) => {
            let fmt = match format.parse::<ExportFormat>() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };
            commands::export::run(fmt)
        }
        Some(Commands::Hotspots { json }) => commands::hotspots::run(json),
        Some(Commands::Reviewers { json }) => commands::reviewers::run(json),
        Some(Commands::Inject {
            directory,
            remove,
            widget_url,
            json,
        }) => commands::inject::run(InjectOptions {
            directory,
            remove,
            widget_url,
            json,
        }),
        Some(Commands::Serve { port, dir, marked }) => commands::serve::run(ServeOptions {
            port,
            directory: dir,
            marked,
        }),
        Some(Commands::Deploy { backend }) => match backend {
            DeployBackend::Cloudflare { dir, json } => {
                commands::deploy::run(DeployOptions { dir, json })
            }
        },
        Some(Commands::Pull {
            endpoint,
            token,
            from,
            json,
        }) => commands::pull::run(PullOptions {
            endpoint,
            token,
            from,
            json,
        }),
        Some(Commands::Push {
            endpoint,
            token,
            json,
        }) => commands::push::run(PushOptions {
            endpoint,
            token,
            json,
        }),
        Some(Commands::Sync { json }) => commands::sync::run(json),
        Some(Commands::Remote { action }) => match action {
            RemoteAction::Add { endpoint, token, hosted } => {
                commands::remote::add(&endpoint, token, hosted)
            }
            RemoteAction::Remove => commands::remote::remove(),
            RemoteAction::Show { json } => commands::remote::show(json),
        },
        Some(Commands::Config { json }) => commands::config_cmd::run(json),
        Some(Commands::Version) => {
            println!("spikes {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(Commands::Login { token, json }) => commands::login::run(LoginOptions { token, json }),
        Some(Commands::Dashboard { json }) => commands::dashboard::run(json),
        Some(Commands::Share { directory, name, json }) => {
            commands::share::run(ShareOptions { directory, name, json })
        }
        Some(Commands::Shares { json }) => commands::shares::run(SharesOptions { json }),
        Some(Commands::Unshare { slug, force, json }) => {
            commands::unshare::run(UnshareOptions { slug, force, json })
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
