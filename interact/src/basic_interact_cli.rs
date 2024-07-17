use clap::{Args, Parser, Subcommand};

/// Crash Interact CLI
#[derive(Default, PartialEq, Eq, Debug, Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct InteractCli {
    #[command(subcommand)]
    pub command: Option<InteractCliCommand>,
}

/// Crash Interact CLI Commands
#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum InteractCliCommand {
    #[command(name = "add", about = "Add value")]
    Add(AddArgs),
    #[command(name = "deploy", about = "Deploy contract")]
    Deploy,
    #[command(name = "feed", about = "Feed contract EGLD")]
    Feed,
    #[command(name = "multi-deploy", about = "Multiple deploy contracts")]
    MultiDeploy(MultiDeployArgs),
    #[command(name = "sum", about = "Print sum")]
    Sum,
    #[command(name = "upgrade", about = "Upgrade contract")]
    Upgrade(UpgradeArgs),

    #[command(name = "deploy-caller", about = "Deploy caller contract")]
    DeployCaller,
    #[command(name = "call-caller", about = "Call caller to add")]
    CallCaller(AddArgs),
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct AddArgs {
    /// The value to add
    #[arg(short = 'v', long = "value", verbatim_doc_comment)]
    pub value: u32,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct UpgradeArgs {
    /// The value to add
    #[arg(short = 'v', long = "value", verbatim_doc_comment)]
    pub value: u32,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct MultiDeployArgs {
    /// The number of contracts to deploy
    #[arg(short = 'c', long = "count", verbatim_doc_comment)]
    pub count: u8,
}
