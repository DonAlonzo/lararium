use cli::Cli;

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value = "lararium")]
    host: String,
    #[arg(env, long, default_value_t = 443)]
    port: u16,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let cli = Cli::connect(args.host, args.port);

    Ok(())
}
