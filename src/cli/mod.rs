use clap::Parser;
use thiserror::Error;

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
    #[command(subcommand)]
    /// Mach-O commands
    Macho(MachoCommand),
    #[command(subcommand)]
    /// ELF commands
    Elf(ElfCommand),
}

#[derive(Debug, Parser)]
pub enum MachoCommand {
    DeleteRpath(delete::Args),
    ChangeRpath(change::Args),
    AddRpath(add::Args),
    ChangeInstallName(install_name::Args),
    ChangeInstallId(install_id::Args),
}

pub fn macho_execute(macho: MachoCommand) -> Result<(), ArwenError> {
    match macho {
        MachoCommand::DeleteRpath(args) => delete::execute(args).map_err(ArwenError::Macho),
        MachoCommand::ChangeRpath(args) => change::execute(args).map_err(ArwenError::Macho),
        MachoCommand::AddRpath(args) => add::execute(args).map_err(ArwenError::Macho),
        MachoCommand::ChangeInstallName(args) => {
            install_name::execute(args).map_err(ArwenError::Macho)
        }
        MachoCommand::ChangeInstallId(args) => install_id::execute(args).map_err(ArwenError::Macho),
    }
}

#[derive(Debug, Parser)]
pub enum ElfCommand {
    AddRpath(elf::add_rpath::Args),
    RemoveRpath(elf::remove_rpath::Args),
    SetRpath(elf::set_rpath::Args),
    PrintRpath(elf::print_rpath::Args),
    ForceRpath(elf::force_rpath::Args),
    SetInterpreter(elf::set_interpreter::Args),
    PrintInterpreter(elf::print_interpreter::Args),
    SetOsAbi(elf::set_os_abi::Args),
    PrintOsAbi(elf::print_os_abi::Args),
    SetSoname(elf::set_soname::Args),
    PrintSoname(elf::print_soname::Args),
    ShrinkRpath(elf::shrink_rpath::Args),
    AddNeeded(elf::add_needed::Args),
    RemoveNeeded(elf::remove_needed::Args),
    ReplaceNeeded(elf::replace_needed::Args),
    PrintNeeded(elf::print_needed::Args),
    NoDefaultLib(elf::no_default_lib::Args),
    ClearSymbolVersion(elf::clear_version_symbol::Args),
    RenameDynamicSymbols(elf::rename_dynamic_symbols::Args),
    AddDebugTag(elf::add_debug_tag::Args),
    ClearExecStack(elf::clear_execstack::Args),
    SetExecStack(elf::set_execstack::Args),
    PrintExecStack(elf::print_execstack::Args),
}

#[derive(Parser, Debug)]
#[command()]
#[clap(arg_required_else_help = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

pub fn execute() -> Result<(), ArwenError> {
    let args = Args::parse();
    match args.command {
        Command::Macho(args) => macho_execute(args),
        Command::Elf(elf) => elf::execute(elf).map_err(ArwenError::Elf),
    }
}

#[derive(Debug, Error)]
pub enum ArwenError {
    #[error("error while patching Mach-O file")]
    Macho(#[from] crate::macho::MachoError),

    #[error("error while patching ELF file")]
    Elf(#[from] crate::elf::ElfError),
}
