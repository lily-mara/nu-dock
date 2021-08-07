use chrono::{DateTime, FixedOffset, NaiveDateTime};
use nu_protocol::{Dictionary, Primitive, UntaggedValue};

/// Replace an i64 representing a UTC timestamp with a nu-enabled Date at the
/// given key
pub fn convert_timestamp(row: &mut Dictionary, key: &str) {
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

/// Replace an i64 with a nu-enabled byte size measurement at the given key
pub fn convert_byte_size(row: &mut Dictionary, key: &str) {
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

/// Remove a column from a dictionary
pub fn remove_column(row: &mut Dictionary, key: &str) {
    row.entries.remove(key);
}
