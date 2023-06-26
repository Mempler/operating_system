use clap::Parser;

mod actions;
mod manifest;
mod resolve;

#[macro_use]
extern crate tracing;

#[derive(Debug, Parser)]
struct AppOpt {
    #[clap(subcommand)]
    action: AppAction,
}

#[derive(Debug, Parser)]
enum AppAction {
    #[clap(name = "build")]
    Build(actions::build::BuildOpt),

    #[clap(name = "run")]
    Run(actions::run::RunOpt),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = AppOpt::parse();
    match opt.action {
        AppAction::Build(opt) => actions::build::perform(opt).await?,
        AppAction::Run(opt) => actions::run::perform(opt).await?,
    };

    Ok(())
}
