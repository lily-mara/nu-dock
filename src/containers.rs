use nu_errors::ShellError;
use nu_plugin::Plugin;
use nu_protocol::{CallInfo, ReturnValue, Signature};

use crate::{run, Mode};

pub struct ContainersCommand;

impl Plugin for ContainersCommand {
    fn config(&mut self) -> Result<Signature, ShellError> {
        Ok(Signature::build("dock containers")
            .desc("View information about docker containers")
            .filter())
    }

    fn begin_filter(&mut self, call_info: CallInfo) -> Result<Vec<ReturnValue>, ShellError> {
        run(Mode::Containers, call_info.name_tag)
    }
}
