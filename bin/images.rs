use bollard::Docker;
use nu_errors::ShellError;
use nu_plugin::Plugin;
use nu_protocol::{CallInfo, ReturnValue, Signature};

use nu_dock::{cleanup, run};

struct Cmd;

impl Plugin for Cmd {
    fn config(&mut self) -> Result<Signature, ShellError> {
        Ok(Signature::build("dock images")
            .desc("View information about docker images")
            .filter())
    }

    fn begin_filter(&mut self, call_info: CallInfo) -> Result<Vec<ReturnValue>, ShellError> {
        run(
            async {
                let docker = Docker::connect_with_local_defaults()?;

                let images = docker.list_images::<String>(None).await?;

                Ok(images)
            },
            call_info.name_tag,
            |row| {
                cleanup::remove_column(row, "VirtualSize");
                cleanup::remove_column(row, "SharedSize");
                cleanup::remove_column(row, "Containers");
                cleanup::remove_column(row, "ParentId");

                cleanup::convert_timestamp(row, "Created");
                cleanup::convert_byte_size(row, "Size");
            },
        )
    }
}
fn main() {
    nu_plugin::serve_plugin(&mut Cmd);
}
