use seelen_core::state::{AppConfig, Placeholder, Settings, Theme, WegItem, WindowManagerLayout};

fn write_schema<T>(path: &str)
where
    T: schemars::JsonSchema,
{
    let schema = schemars::schema_for!(T);
    std::fs::write(path, serde_json::to_string_pretty(&schema).unwrap()).unwrap();
}

fn main() {
    write_schema::<Settings>("./dist/settings.schema.json");
    write_schema::<Placeholder>("./dist/placeholder.schema.json");
    write_schema::<Theme>("./dist/theme.schema.json");
    write_schema::<WindowManagerLayout>("./dist/layout.schema.json");
    write_schema::<Vec<AppConfig>>("./dist/settings_by_app.schema.json");
    write_schema::<Vec<WegItem>>("./dist/weg_items.schema.json");
}
