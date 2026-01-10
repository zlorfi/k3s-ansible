use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use tracing::info;

mod mikrotik;
use mikrotik::MikroTikClient;

// Load environment variables from .env file
fn load_env() {
    dotenv::dotenv().ok();
}

#[derive(Parser)]
#[command(name = "mikrotik")]
#[command(about = "MikroTik RouterOS PoE Control CLI", long_about = None)]
#[command(version)]
#[command(author)]
struct Cli {
    /// MikroTik device IP/hostname
    #[arg(
        short,
        long,
        env = "MIKROTIK_HOST",
        default_value = "192.168.1.1",
        global = true
    )]
    host: String,

    /// API username
    #[arg(
        short,
        long,
        env = "MIKROTIK_USER",
        default_value = "admin",
        global = true
    )]
    user: String,

    /// API password
    #[arg(short, long, env = "MIKROTIK_PASSWORD", global = true)]
    password: Option<String>,

    /// Connection timeout in seconds
    #[arg(
        short = 't',
        long,
        env = "MIKROTIK_TIMEOUT",
        default_value = "10",
        global = true
    )]
    timeout: u64,

    /// API port number
    #[arg(long, env = "MIKROTIK_PORT", default_value = "8728", global = true)]
    port: u16,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enable PoE on the MikroTik device
    On,

    /// Disable PoE on the MikroTik device
    Off,

    /// Check PoE status on the MikroTik device
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    load_env();

    let cli = Cli::parse();

    // Initialize tracing
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_writer(std::io::stderr)
            .init();
    }

    // Validate password
    let password = cli.password.ok_or_else(|| {
        anyhow!("Password required. Use --password or set MIKROTIK_PASSWORD env var")
    })?;

    if cli.verbose {
        info!("Configuration:");
        info!("  Host: {}", cli.host);
        info!("  Port: {}", cli.port);
        info!("  User: {}", cli.user);
        info!("  Timeout: {}s", cli.timeout);
    }

    // Create MikroTik client
    let mut client =
        MikroTikClient::new(&cli.host, cli.port, &cli.user, &password, cli.timeout).await?;

    // Execute command
    match cli.command {
        Commands::On => {
            info!("Connecting to MikroTik device at {}", cli.host);
            info!("Executing 'poe_on' script...");

            client.run_script("poe_on").await?;

            println!("✓ PoE enabled successfully");
            info!("Script executed successfully");
        }
        Commands::Off => {
            info!("Connecting to MikroTik device at {}", cli.host);
            info!("Executing 'poe_off' script...");

            client.run_script("poe_off").await?;

            println!("✓ PoE disabled successfully");
            info!("Script executed successfully");
        }
        Commands::Status => {
            info!("Connecting to MikroTik device at {}", cli.host);
            info!("Fetching PoE status...");

            let status = client.get_poe_status().await?;

            println!("\n{}", "=".repeat(70));
            println!("MikroTik PoE Status");
            println!("{}", "=".repeat(70));
            println!("Device: {}", cli.host);
            println!("User: {}", cli.user);
            println!("{}", "-".repeat(70));
            println!("{}", status);
            println!("{}\n", "=".repeat(70));
        }
    }

    Ok(())
}
