use bollard::Docker;
use nu_errors::ShellError;
use nu_plugin::Plugin;
use nu_protocol::{CallInfo, ReturnValue, Signature};

use nu_dock::{cleanup, run};

pub struct Cmd;

impl Plugin for Cmd {
    fn config(&mut self) -> Result<Signature, ShellError> {
        Ok(Signature::build("dock containers")
            .desc("View information about docker containers")
            .filter())
    }

    fn begin_filter(&mut self, call_info: CallInfo) -> Result<Vec<ReturnValue>, ShellError> {
        run(
            async {
                let docker = Docker::connect_with_local_defaults()?;

                let containers = docker.list_containers::<String>(None).await?;

                Ok(containers)
            },
            call_info.name_tag,
            |row| {
                cleanup::convert_timestamp(row, "Created");
                cleanup::convert_byte_size(row, "Size");
            },
        )
    }
}

fn main() {
    nu_plugin::serve_plugin(&mut Cmd);
}
