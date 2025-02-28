use clap::Parser;

use crate::macho::MachoError;

/// Macho CLI
pub mod add;
pub mod change;
pub mod delete;
pub mod install_id;
pub mod install_name;

/// ELF CLI
pub mod elf;

#[derive(Parser, Debug)]
pub enum Command {
    DeleteRpath(delete::Args),
    ChangeRpath(change::Args),
    AddRpath(add::Args),
    ChangeInstallName(install_name::Args),
    ChangeInstallId(install_id::Args),
    #[command(subcommand)]
    Elf(ElfCommand),
}

#[derive(Debug, Parser)]
pub enum ElfCommand {
    AddRpath(elf::add_rpath::Args),
    RemoveRpath(elf::remove_rpath::Args),
    SetRpath(elf::set_rpath::Args),
    ForceRpath(elf::force_rpath::Args),
    SetInterpreter(elf::set_interpreter::Args),
    SetOsAbi(elf::set_os_abi::Args),
    SetSoname(elf::set_soname::Args),
    ShrinkRpath(elf::shrink_rpath::Args),
    AddNeeded(elf::add_needed::Args),
    RemoveNeeded(elf::remove_needed::Args),
    ReplaceNeeded(elf::replace_needed::Args),
    NoDefaultLib(elf::no_default_lib::Args),
    ClearVersionSymbol(elf::clear_version_symbol::Args),
    AddDebugTag(elf::add_debug_tag::Args),
    ClearExecStack(elf::clear_execstack::Args),
    SetExecStack(elf::set_execstack::Args),
}

#[derive(Parser, Debug)]
#[command()]
#[clap(arg_required_else_help = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

pub fn execute() -> Result<(), MachoError> {
    let args = Args::parse();
    match args.command {
        Command::DeleteRpath(args) => delete::execute(args),
        Command::ChangeRpath(args) => change::execute(args),
        Command::AddRpath(args) => add::execute(args),
        Command::ChangeInstallName(args) => install_name::execute(args),
        Command::ChangeInstallId(args) => install_id::execute(args),
        Command::Elf(elf) => elf::execute(elf),
    }
}
