/* use schemars::schema_for;
use seelen_core::state::Settings;

fn save_schema(name: &str, schema: &schemars::schema::RootSchema) {
    std::fs::write(
        format!("documentation/schemas/{}.schema.json", name),
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap()
}
 */
fn main() {
    tauri_build::build();
    // save_schema("settings", &schema_for!(Settings));
}
