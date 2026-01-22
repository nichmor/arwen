pub mod add;
pub mod change;
pub mod delete;
pub mod install_id;
pub mod install_name;

use super::MachoCommand;
use arwen_macho::MachoError;

pub fn execute(macho: MachoCommand) -> Result<(), MachoError> {
    match macho {
        MachoCommand::DeleteRpath(args) => delete::execute(args),
        MachoCommand::ChangeRpath(args) => change::execute(args),
        MachoCommand::AddRpath(args) => add::execute(args),
        MachoCommand::ChangeInstallName(args) => install_name::execute(args),
        MachoCommand::ChangeInstallId(args) => install_id::execute(args),
    }
}
