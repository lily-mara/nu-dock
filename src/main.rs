use bollard::{
    models::{ContainerSummaryInner, ImageSummary},
    Docker,
};
use nu_errors::ShellError;
use nu_plugin::{serve_plugin, Plugin};
use nu_protocol::{CallInfo, ReturnValue, Signature, SyntaxShape};
use nu_source::HasSpan;

#[derive(serde::Serialize)]
#[serde(untagged)]
enum Value {
    Container(ContainerSummaryInner),
    Image(ImageSummary),
}

enum Mode {
    Containers,
    Images,
}

struct Implementation {}

impl Implementation {
    fn new() -> Self {
        Self {}
    }

    async fn do_begin_filter(&mut self, mode: Mode) -> Result<Vec<Value>, anyhow::Error> {
        let docker = Docker::connect_with_local_defaults()?;

        let values = match mode {
            Mode::Containers => docker
                .list_containers::<String>(None)
                .await?
                .into_iter()
                .map(Value::Container)
                .collect(),
            Mode::Images => docker
                .list_images::<String>(None)
                .await?
                .into_iter()
                .map(Value::Image)
                .collect(),
        };

        Ok(values)
    }
}

impl Plugin for Implementation {
    fn config(&mut self) -> Result<Signature, ShellError> {
        Ok(Signature::build("dock")
            .desc("View information about docker images and containers")
            .required(
                "COMMAND",
                SyntaxShape::String,
                "either 'images' or 'containers'",
            )
            .filter())
    }

    fn begin_filter(&mut self, call_info: CallInfo) -> Result<Vec<ReturnValue>, ShellError> {
        let mode_value = call_info.args.expect_nth(0)?;

        let mode = match mode_value.expect_string() {
            "images" => Mode::Images,
            "containers" => Mode::Containers,
            _ => {
                return Err(ShellError::labeled_error(
                    "Invalid subcommand. Expected images or containers",
                    "invalid subcommand",
                    mode_value.span(),
                ));
            }
        };

        let rt = tokio::runtime::Runtime::new().unwrap();

        let rows = rt.block_on(self.do_begin_filter(mode)).map_err(|e| {
            ShellError::labeled_error(format!("dock failed: {}", e), "error", &call_info.name_tag)
        })?;

        let values =
            serde_nu::to_success_return_values(rows, &call_info.name_tag).map_err(|e| {
                ShellError::labeled_error(
                    format!("failed to convert output values: {}", e),
                    "failed to convert output values",
                    &call_info.name_tag,
                )
            })?;

        Ok(values)
    }
}

fn main() {
    serve_plugin(&mut Implementation::new());
}
