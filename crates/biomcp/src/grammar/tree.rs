//! Keep declaration help beside its one built parser snapshot.
use biomcp_types::HelpCommand;
use clap::Command;

pub(crate) struct Root {
    command: Command,
    housekeeping: Vec<HelpCommand>,
}
impl Root {
    pub(crate) fn new(mut command: Command) -> Self {
        let housekeeping = crate::housekeeping::help(&command);
        command.build();
        Self {
            command,
            housekeeping,
        }
    }
    pub(crate) fn housekeeping(&self) -> &[HelpCommand] {
        &self.housekeeping
    }
}
impl std::ops::Deref for Root {
    type Target = Command;
    fn deref(&self) -> &Self::Target {
        &self.command
    }
}
