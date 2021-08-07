use nu_errors::ShellError;
use nu_plugin::Plugin;
use nu_protocol::{CallInfo, ReturnValue, Signature};

use crate::{run, Mode};

pub struct ImagesCommand;

impl Plugin for ImagesCommand {
    fn config(&mut self) -> Result<Signature, ShellError> {
        Ok(Signature::build("dock images")
            .desc("View information about docker images")
            .filter())
    }

    fn begin_filter(&mut self, call_info: CallInfo) -> Result<Vec<ReturnValue>, ShellError> {
        run(Mode::Images, call_info.name_tag)
    }
}
