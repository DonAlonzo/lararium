use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _args = Args::parse();
    Ok(())
}
