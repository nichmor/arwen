use clap::Parser;
use thiserror::Error;

pub mod elf;
pub mod macho;

#[derive(Parser, Debug)]
/// The `arwen`
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
    DeleteRpath(macho::delete::Args),
    ChangeRpath(macho::change::Args),
    AddRpath(macho::add::Args),
    ChangeInstallName(macho::install_name::Args),
    ChangeInstallId(macho::install_id::Args),
    AdhocSign(macho::codesign::Args),
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
    SetPageSize(elf::set_page_size::Args),
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
        Command::Macho(args) => macho::execute(args).map_err(ArwenError::Macho),
        Command::Elf(elf) => elf::execute(elf).map_err(ArwenError::Elf),
    }
}

#[derive(Debug, Error)]
pub enum ArwenError {
    #[error("error while patching Mach-O file")]
    Macho(#[from] arwen_macho::MachoError),

    #[error("error while patching ELF file")]
    Elf(#[from] arwen_elf::ElfError),
}
