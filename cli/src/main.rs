mod api;
mod auth;
mod commands;
mod config;
mod error;
mod output;
mod spike;
mod state;
mod storage;

use clap::{Parser, Subcommand};
use commands::auth_keys::CreateKeyOptions;
use commands::delete::DeleteOptions;
use commands::deploy::DeployOptions;
use commands::export::ExportFormat;
use commands::inject::InjectOptions;
use commands::list::ListOptions;
use commands::login::LoginOptions;
use commands::pull::PullOptions;
use commands::push::PushOptions;
use commands::reply::ReplyOptions;
use commands::resolve::ResolveOptions;
use commands::serve::ServeOptions;
use commands::share::ShareOptions;
use commands::shares::SharesOptions;
use commands::unshare::UnshareOptions;
use commands::usage::UsageOptions;
use commands::watch::WatchOptions;

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

        /// Self-host instead of using hosted spikes.sh (non-interactive opt-out)
        #[arg(long)]
        self_host: bool,
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

        /// Show only unresolved spikes
        #[arg(long)]
        unresolved: bool,
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

        /// Endpoint URL for the widget to POST feedback to (overrides config)
        #[arg(long)]
        endpoint: Option<String>,

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

        /// Allowed CORS origin (e.g., https://spikes.sh). Without this flag, CORS is disabled (same-origin only).
        #[arg(long)]
        cors_allow_origin: Option<String>,
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

        /// Only spikes updated after this ISO 8601 timestamp, or "last" for the previous pull
        #[arg(long)]
        since: Option<String>,

        /// Only spikes whose URL starts with this prefix
        #[arg(long)]
        url_prefix: Option<String>,

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

    /// Update spikes CLI and widget to the latest version
    Update,

    /// Log in to spikes.sh hosted service
    Login {
        /// Auth token (or enter interactively)
        #[arg(long)]
        token: Option<String>,

        /// Use email magic link instead of browser flow
        #[arg(long)]
        email: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Log out from spikes.sh
    Logout {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show current user identity
    Whoami {
        /// Output as JSON
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

        /// Password-protect the share (Pro only)
        #[arg(long)]
        password: Option<String>,

        /// Host URL for the API
        #[arg(long, default_value = "https://spikes.sh")]
        host: String,

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

    /// Delete a spike from local storage
    Delete {
        /// Spike ID or prefix (minimum 4 characters)
        id: String,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Mark a spike as addressed, won't do, or open again
    Resolve {
        /// Spike ID or prefix (minimum 4 characters)
        id: String,

        /// Mark as unresolved instead (alias of --undo)
        #[arg(long)]
        unresolve: bool,

        /// Reopen the spike (status: open)
        #[arg(long)]
        undo: bool,

        /// Version label the spike was addressed in (status: addressed)
        #[arg(long = "in", value_name = "LABEL")]
        addressed_in: Option<String>,

        /// Mark as won't do (status: wont_do)
        #[arg(long)]
        wont_do: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Reply to a spike (hosted), optionally changing its status
    Reply {
        /// Spike ID or prefix
        id: String,

        /// Reply text
        text: String,

        /// Version label this reply refers to (implies status: addressed)
        #[arg(long, value_name = "LABEL")]
        version: Option<String>,

        /// Set the spike status: addressed, wont_do, or open
        #[arg(long)]
        status: Option<String>,

        /// Author name shown to the reviewer (default: Builder)
        #[arg(long)]
        name: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show which credential is in use, whether it works, and cache freshness
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage hosted projects
    Projects {
        #[command(subcommand)]
        action: ProjectsAction,
    },

    /// Declare review versions for the configured project
    Versions {
        #[command(subcommand)]
        action: VersionsAction,
    },

    /// Ask reviewers questions and read their answers
    Questions {
        #[command(subcommand)]
        action: QuestionsAction,
    },

    /// Stream new and updated feedback as JSON lines
    Watch {
        /// ISO 8601 timestamp or "last" (default: last pull, else now)
        #[arg(long)]
        since: Option<String>,

        /// Poll interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,

        /// Only spikes whose URL starts with this prefix
        #[arg(long)]
        url_prefix: Option<String>,

        /// Run this shell command per event with the JSON on stdin
        #[arg(long, value_name = "COMMAND")]
        exec: Option<String>,

        /// Poll once and exit
        #[arg(long)]
        once: bool,
    },

    /// Open Stripe billing portal in browser
    Billing {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Upgrade to Pro subscription via Stripe checkout
    Upgrade {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Display current usage statistics
    Usage {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// MCP (Model Context Protocol) server for agent integration
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },

    /// Manage API keys for agent authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Subcommand)]
enum DeployBackend {
    /// Scaffold Cloudflare Worker + D1 for self-hosted feedback sync
    ///
    /// This scaffolds a Cloudflare Worker with D1 database for self-hosting your feedback
    /// backend. Use this when you need data isolation or a custom domain. For quick
    /// temporary previews, use `spikes share` instead.
    ///
    /// spikes.sh already hosts this backend for you — this is for self-hosting only.
    Cloudflare {
        /// Output directory (default: ./spikes-worker)
        #[arg(long)]
        dir: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Bypass the hosted spikes.sh warning prompt (non-interactive)
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum McpAction {
    /// Start the MCP server (stdio transport by default)
    Serve {
        /// Use hosted API instead of local JSONL (requires SPIKES_TOKEN or auth.toml)
        #[arg(long)]
        remote: bool,

        /// Transport mode: stdio or http (default: stdio)
        #[arg(long, value_enum, default_value = "stdio")]
        transport: McpTransport,

        /// Port for HTTP transport (default: 3848, only used with --transport http)
        #[arg(long, default_value = "3848")]
        port: u16,

        /// Bind address for HTTP transport (default: 127.0.0.1, only used with --transport http)
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
    },

    /// Generate MCP config for Claude Desktop, Cursor, or other clients
    Install {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// MCP transport mode
#[derive(Clone, Debug, clap::ValueEnum)]
enum McpTransport {
    /// Use standard input/output for JSON-RPC (default)
    Stdio,
    /// Use HTTP transport with POST endpoint
    Http,
}

#[derive(Subcommand)]
enum AuthAction {
    /// Create a new API key for agent authentication
    CreateKey {
        /// Optional name/label for the key
        #[arg(long)]
        name: Option<String>,

        /// Scope the key to one project (requires spikes login)
        #[arg(long, value_name = "KEY")]
        project: Option<String>,

        /// Write the key into .spikes/config.toml under [remote]
        #[arg(long)]
        save: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List all API keys
    ListKeys {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Revoke an API key
    RevokeKey {
        /// The key ID to revoke (e.g. key_abc123)
        key_id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ProjectsAction {
    /// Create a project on the hosted service
    Create {
        /// Project key (lowercase letters, digits, hyphens)
        key: String,

        /// Allowed origin (repeatable), e.g. https://example.com
        #[arg(long = "origin", value_name = "ORIGIN")]
        origins: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List your projects
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum VersionsAction {
    /// Add a version (label + URL prefix)
    Add {
        /// Version label, e.g. v0.5
        label: String,

        /// URL prefix that identifies this version, e.g. /prosser/versions/v0-5/
        #[arg(long, value_name = "URL_PREFIX")]
        prefix: String,

        /// Notes shown to reviewers (what changed)
        #[arg(long)]
        notes: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List versions with spike counts
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set the notes of a version
    Notes {
        /// Version label
        label: String,

        /// Notes text
        text: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum QuestionsAction {
    /// Ask reviewers a question
    Add {
        /// Question title
        title: String,

        /// Longer question body
        #[arg(long)]
        body: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List questions (open by default)
    List {
        /// Show closed questions instead
        #[arg(long)]
        closed: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show the answers to a question
    Answers {
        /// Question ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Close a question
    Close {
        /// Question ID
        id: String,

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
        Some(Commands::Init { json, self_host }) => commands::init::run(json, self_host),
        Some(Commands::List {
            json,
            page,
            reviewer,
            rating,
            unresolved,
        }) => commands::list::run(ListOptions {
            json,
            page,
            reviewer,
            rating,
            unresolved,
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
            endpoint,
            json,
        }) => commands::inject::run(InjectOptions {
            directory,
            remove,
            widget_url,
            endpoint,
            json,
        }),
        Some(Commands::Serve {
            port,
            dir,
            marked,
            cors_allow_origin,
        }) => commands::serve::run(ServeOptions {
            port,
            directory: dir,
            marked,
            cors_allow_origin,
        }),
        Some(Commands::Deploy { backend }) => match backend {
            DeployBackend::Cloudflare { dir, json, force } => {
                commands::deploy::run(DeployOptions { dir, json, force })
            }
        },
        Some(Commands::Pull {
            endpoint,
            token,
            from,
            since,
            url_prefix,
            json,
        }) => commands::pull::run(PullOptions {
            endpoint,
            token,
            from,
            since,
            url_prefix,
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
            RemoteAction::Add {
                endpoint,
                token,
                hosted,
            } => commands::remote::add(&endpoint, token, hosted),
            RemoteAction::Remove => commands::remote::remove(),
            RemoteAction::Show { json } => commands::remote::show(json),
        },
        Some(Commands::Config { json }) => commands::config_cmd::run(json),
        Some(Commands::Version) => {
            println!("spikes {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(Commands::Update) => commands::update::run(),
        Some(Commands::Login { token, email, json }) => {
            commands::login::run(LoginOptions { token, email, json })
        }
        Some(Commands::Logout { json }) => commands::logout::run(json),
        Some(Commands::Whoami { json }) => commands::whoami::run(json),
        Some(Commands::Share {
            directory,
            name,
            password,
            host,
            json,
        }) => commands::share::run(ShareOptions {
            directory,
            name,
            password,
            host,
            json,
        }),
        Some(Commands::Shares { json }) => commands::shares::run(SharesOptions { json }),
        Some(Commands::Unshare { slug, force, json }) => {
            commands::unshare::run(UnshareOptions { slug, force, json })
        }
        Some(Commands::Delete { id, force, json }) => {
            commands::delete::run(DeleteOptions { id, force, json })
        }
        Some(Commands::Resolve {
            id,
            unresolve,
            undo,
            addressed_in,
            wont_do,
            json,
        }) => commands::resolve::run(ResolveOptions {
            id,
            unresolve,
            undo,
            addressed_in,
            wont_do,
            json,
        }),
        Some(Commands::Reply {
            id,
            text,
            version,
            status,
            name,
            json,
        }) => commands::reply::run(ReplyOptions {
            id,
            text,
            version,
            status,
            name,
            json,
        }),
        Some(Commands::Status { json }) => commands::status::run(json),
        Some(Commands::Projects { action }) => match action {
            ProjectsAction::Create { key, origins, json } => {
                commands::projects::create(&key, origins, json)
            }
            ProjectsAction::List { json } => commands::projects::list(json),
        },
        Some(Commands::Versions { action }) => match action {
            VersionsAction::Add {
                label,
                prefix,
                notes,
                json,
            } => commands::versions::add(&label, &prefix, notes, json),
            VersionsAction::List { json } => commands::versions::list(json),
            VersionsAction::Notes { label, text, json } => {
                commands::versions::notes(&label, &text, json)
            }
        },
        Some(Commands::Questions { action }) => match action {
            QuestionsAction::Add { title, body, json } => {
                commands::questions::add(&title, body, json)
            }
            QuestionsAction::List { closed, json } => commands::questions::list(closed, json),
            QuestionsAction::Answers { id, json } => commands::questions::answers(&id, json),
            QuestionsAction::Close { id, json } => commands::questions::close(&id, json),
        },
        Some(Commands::Watch {
            since,
            interval,
            url_prefix,
            exec,
            once,
        }) => commands::watch::run(WatchOptions {
            since,
            interval,
            url_prefix,
            exec,
            once,
        }),
        Some(Commands::Billing { json }) => commands::billing::run(json),
        Some(Commands::Upgrade { json }) => commands::upgrade::run(json),
        Some(Commands::Usage { json }) => commands::usage::run(UsageOptions { json }),
        Some(Commands::Mcp { action }) => match action {
            McpAction::Serve {
                remote,
                transport,
                port,
                bind,
            } => {
                let transport_mode = match transport {
                    McpTransport::Stdio => commands::mcp::TransportMode::Stdio,
                    McpTransport::Http => commands::mcp::TransportMode::Http { port, bind },
                };
                commands::mcp::run(remote, transport_mode)
            }
            McpAction::Install { json } => commands::mcp::install(json),
        },
        Some(Commands::Auth { action }) => match action {
            AuthAction::CreateKey {
                name,
                project,
                save,
                json,
            } => commands::auth_keys::create_key(CreateKeyOptions {
                name,
                project,
                save,
                json,
            }),
            AuthAction::ListKeys { json } => commands::auth_keys::list_keys(json),
            AuthAction::RevokeKey { key_id, json } => {
                commands::auth_keys::revoke_key(&key_id, json)
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
