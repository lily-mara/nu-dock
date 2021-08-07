use nu_plugin::Plugin;
use nu_protocol::{ReturnSuccess, ReturnValue, Signature, UntaggedValue};
use nu_source::Tag;

struct Dock;

fn main() {
    nu_plugin::serve_plugin(&mut Dock);
}

impl Dock {
    fn usage(&mut self) -> String {
        format!(
            "{}\nRun with --help for details on subcommands",
            self.config().unwrap().usage
        )
    }
}

impl Plugin for Dock {
    fn config(&mut self) -> Result<Signature, nu_errors::ShellError> {
        Ok(Signature::build("dock")
            .desc("View information about docker images and containers")
            .filter())
    }

    fn begin_filter(
        &mut self,
        _call_info: nu_protocol::CallInfo,
    ) -> Result<Vec<ReturnValue>, nu_errors::ShellError> {
        Ok(vec![Ok(ReturnSuccess::Value(
            UntaggedValue::string(self.usage()).into_value(Tag::unknown()),
        ))])
    }
}
