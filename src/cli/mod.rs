use clap::Parser;

pub mod add;
pub mod change;
pub mod delete;
pub mod install_id;
pub mod install_name;

#[derive(Parser, Debug)]
pub enum Command {
    DeleteRpath(delete::Args),
    ChangeRpath(change::Args),
    AddRpath(add::Args),
    ChangeInstallName(install_name::Args),
    ChangeInstallId(install_id::Args),
}

#[derive(Parser, Debug)]
#[command()]
#[clap(arg_required_else_help = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

pub fn execute() {
    let args = Args::parse();
    match args.command {
        Command::DeleteRpath(args) => {
            delete::execute(args);
        }
        Command::ChangeRpath(args) => {
            change::execute(args);
        }
        Command::AddRpath(args) => {
            add::execute(args);
        }
        Command::ChangeInstallName(args) => {
            install_name::execute(args);
        }
        Command::ChangeInstallId(args) => {
            install_id::execute(args);
        }
    }
}
