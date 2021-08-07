use std::future::Future;

use nu_errors::ShellError;
use nu_protocol::{Dictionary, ReturnSuccess, ReturnValue, UntaggedValue};
use nu_source::Tag;

pub mod cleanup;

pub fn run<F, O, I, C>(fut: F, tag: Tag, cleanup: C) -> Result<Vec<ReturnValue>, ShellError>
where
    F: Future<Output = Result<I, anyhow::Error>>,
    O: serde::Serialize,
    I: IntoIterator<Item = O>,
    C: Fn(&mut Dictionary),
{
    let rt = tokio::runtime::Runtime::new().unwrap();

    let rows = rt
        .block_on(fut)
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
                if let UntaggedValue::Row(row) = &mut v.value {
                    cleanup(row);
                }
            }
        }
    }

    Ok(values)
}
