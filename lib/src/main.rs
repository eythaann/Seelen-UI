use seelen_core::state::{Placeholder, Settings, Theme};

fn write_schema<T>(filename: &str)
where
    T: schemars::JsonSchema,
{
    let schema = schemars::schema_for!(T);
    std::fs::write(filename, serde_json::to_string_pretty(&schema).unwrap()).unwrap();
}

fn main() {
    write_schema::<Settings>("settings.schema.json");
    write_schema::<Placeholder>("placeholder.schema.json");
    write_schema::<Theme>("theme.schema.json");
}
