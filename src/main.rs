use bollard::{
    models::{ContainerSummaryInner, ImageSummary},
    Docker,
};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use nu_errors::ShellError;
use nu_plugin::{serve_plugin, Plugin};
use nu_protocol::{
    CallInfo, Dictionary, Primitive, ReturnSuccess, ReturnValue, Signature, SyntaxShape,
    UntaggedValue,
};
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

fn convert_timestamp(row: &mut Dictionary, key: &str) {
    let data = match row.get_mut_data_by_key(key) {
        Some(d) => d,
        None => return,
    };

    let timestamp = match data.value {
        UntaggedValue::Primitive(Primitive::Int(timestamp)) => timestamp,
        _ => return,
    };

    let date_time = DateTime::from_utc(
        NaiveDateTime::from_timestamp(timestamp, 0),
        FixedOffset::east(0),
    );

    data.value = UntaggedValue::Primitive(Primitive::Date(date_time));
}

fn convert_byte_size(row: &mut Dictionary, key: &str) {
    let data = match row.get_mut_data_by_key(key) {
        Some(d) => d,
        None => return,
    };

    let size = match data.value {
        UntaggedValue::Primitive(Primitive::Int(size)) => size as u64,
        _ => return,
    };

    data.value = UntaggedValue::Primitive(Primitive::Filesize(size));
}

fn row_cleanup(value: &mut nu_protocol::Value) {
    let row = match &mut value.value {
        UntaggedValue::Row(row) => row,
        _ => return,
    };

    convert_timestamp(row, "Created");
    convert_byte_size(row, "Size");
    convert_byte_size(row, "VirtualSize");
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

        let mut values =
            serde_nu::to_success_return_values(rows, &call_info.name_tag).map_err(|e| {
                ShellError::labeled_error(
                    format!("failed to convert output values: {}", e),
                    "failed to convert output values",
                    &call_info.name_tag,
                )
            })?;

        for result in &mut values {
            if let Ok(success) = result {
                if let ReturnSuccess::Value(v) = success {
                    row_cleanup(v);
                }
            }
        }

        Ok(values)
    }
}

fn main() {
    serve_plugin(&mut Implementation::new());
}
