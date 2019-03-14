use tera::Value;

pub fn markdown(
    value: Value,
    _: std::collections::HashMap<String, Value>
) -> tera::Result<Value> {

    let input = tera::try_get_value!("markdown", "value", String, value);
    let output = crate::markdown(&input);

    Ok(tera::to_value(output).unwrap())
}
