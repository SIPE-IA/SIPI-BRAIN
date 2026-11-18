use clap::{Parser, Subcommand};
use sipibrain::{api, auth, config, db, pilot};
use std::{
    io::{self, Write},
    net::SocketAddr,
    path::PathBuf,
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "sipibrain",
    version,
    about = "Local synthetic SIPI-brain pilot. Never use real sensitive data."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Seed fictitious pilot knowledge only; never creates usable user credentials.
    SeedPilot {
        #[arg(long, required = true)]
        synthetic: bool,
    },
    /// Initialize the local SQLite schema.
    InitDb,
    /// Interactively create the first administrator (only once).
    BootstrapAdmin,
    /// Create a programmer; password is prompted, never accepted as an argument.
    CreateUser {
        #[arg(long)]
        username: String,
    },
    /// Serve API and compiled Vue bundle on loopback only.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: SocketAddr,
        #[arg(long, default_value = "frontend/dist")]
        static_dir: PathBuf,
    },
}
#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
async fn run(cli: Cli) -> Result<(), String> {
    if matches!(cli.command, Command::SeedPilot { synthetic: false }) {
        return Err("Synthetic seeding requires --synthetic.".into());
    }
    if let Command::Serve {
        bind,
        ref static_dir,
    } = cli.command
    {
        config::validate_bind(bind)?;
        if !static_dir.is_dir() || !static_dir.join("index.html").is_file() {
            return Err("Static directory must contain index.html.".into());
        }
    }
    let dir = config::data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|_| "Cannot create private data directory.")?;
    let pool = db::open(&dir.join("sipibrain.db"))
        .await
        .map_err(|_| "Database initialization failed.")?;
    match cli.command {
        Command::InitDb => println!("Database initialized."),
        Command::SeedPilot { .. } => {
            pilot::seed(&pool).await.map_err(
                |_| "Synthetic seed failed; existing contributions were not overwritten.",
            )?;
            println!("Synthetic pilot ready. User assignments remain administrator-managed.");
            println!(
                "Wikis: {} (SIPI Insight), {} (SIPI Estimador)",
                pilot::INSIGHT,
                pilot::ESTIMADOR
            );
            println!("Projects: {} (Alfa), {} (Beta)", pilot::ALFA, pilot::BETA);
            println!("Clients: pilot-client-alfa, pilot-client-beta");
        }
        Command::BootstrapAdmin | Command::CreateUser { .. } => {
            let admin = matches!(cli.command, Command::BootstrapAdmin);
            let username = match cli.command {
                Command::CreateUser { username } => username,
                _ => {
                    print!("Administrator username: ");
                    io::stdout().flush().map_err(|_| "Cannot prompt.")?;
                    let mut name = String::new();
                    io::stdin()
                        .read_line(&mut name)
                        .map_err(|_| "Cannot read username.")?;
                    name.trim().to_owned()
                }
            };
            let password = rpassword::prompt_password("Password: ")
                .map_err(|_| "Cannot read password from terminal.")?;
            let confirmation = rpassword::prompt_password("Confirm password: ")
                .map_err(|_| "Cannot read password from terminal.")?;
            if password != confirmation {
                return Err("Passwords do not match.".into());
            }
            auth::create_user(&pool, &username, password, admin)
                .await
                .map_err(|_| "User creation rejected; check username and administrator state.")?;
            println!("User created.");
        }
        Command::Serve { bind, static_dir } => {
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .map_err(|_| "Cannot open loopback listener.")?;
            let bound = listener
                .local_addr()
                .map_err(|_| "Cannot resolve listener address.")?;
            let origin = config::origin(bound)?;
            #[cfg(not(feature = "local-connection-fixture"))]
            let app = api::application(pool.clone(), &origin, &static_dir).await?;
            #[cfg(feature = "local-connection-fixture")]
            let app = {
                let catalog = if let Some(path) = std::env::var_os("SIPIBRAIN_LOCAL_FIXTURE_CATALOG") {
                    let path = std::path::PathBuf::from(path);
                    if bound.ip() != std::net::Ipv4Addr::LOCALHOST || !path.is_absolute()
                        || path.to_string_lossy().starts_with(r"\\")
                    {
                        return Err("Local test catalog requires a local absolute path and a 127.0.0.1 listener.".into());
                    }
                    sipibrain::agent_connection::Catalog::fixture(path)
                } else {
                    sipibrain::agent_connection::Catalog::default()
                };
                api::application_with_catalog(pool.clone(), &origin, &static_dir, catalog).await?
            };
            eprintln!("Synthetic data only. Local server: {origin}");
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await
                .map_err(|_| "Server stopped with an error.")?;
        }
    }
    pool.close().await;
    Ok(())
}
