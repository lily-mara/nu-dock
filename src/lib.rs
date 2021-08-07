use bollard::{
    models::{ContainerSummaryInner, ImageSummary},
    Docker,
};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use nu_errors::ShellError;
use nu_protocol::{Dictionary, Primitive, ReturnSuccess, ReturnValue, UntaggedValue};
use nu_source::Tag;

mod containers;
mod images;

pub use containers::ContainersCommand;
pub use images::ImagesCommand;

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

async fn do_begin_filter(mode: Mode) -> Result<Vec<Value>, anyhow::Error> {
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

fn run(mode: Mode, tag: Tag) -> Result<Vec<ReturnValue>, ShellError> {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let rows = rt
        .block_on(do_begin_filter(mode))
        .map_err(|e| ShellError::labeled_error(format!("dock failed: {}", e), "error", &tag))?;

    let mut values = serde_nu::to_success_return_values(rows, &tag).map_err(|e| {
        ShellError::labeled_error(
            format!("failed to convert output values: {}", e),
            "failed to convert output values",
            &tag,
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

fn remove_column(row: &mut Dictionary, key: &str) {
    row.entries.remove(key);
}

fn row_cleanup(value: &mut nu_protocol::Value) {
    let row = match &mut value.value {
        UntaggedValue::Row(row) => row,
        _ => return,
    };

    convert_timestamp(row, "Created");
    convert_byte_size(row, "Size");
    remove_column(row, "VirtualSize");
    remove_column(row, "SharedSize");
    remove_column(row, "Containers");
    remove_column(row, "ParentId");
}
