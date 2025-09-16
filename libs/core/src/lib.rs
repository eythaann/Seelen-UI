pub mod error;
pub mod handlers;
pub mod rect;
pub mod resource;
pub mod state;
pub mod system_state;
pub mod utils;

pub use error::SeelenLibError;

#[macro_use(Serialize, Deserialize)]
extern crate serde;

#[macro_use(JsonSchema)]
extern crate schemars;

#[macro_use(TS)]
extern crate ts_rs;

#[macro_use(FromPrimitive, IntoPrimitive)]
extern crate num_enum;

#[cfg(feature = "gen-binds")]
#[test]
fn generate_schemas() {
    use state::{AppConfig, IconPack, Placeholder, Plugin, Settings, Theme, WegItems, Widget};

    fn write_schema<T>(path: &str)
    where
        T: schemars::JsonSchema,
    {
        let schema = schemars::schema_for!(T);
        std::fs::write(path, serde_json::to_string_pretty(&schema).unwrap()).unwrap();
    }

    std::fs::create_dir_all("./gen/schemas").unwrap();
    write_schema::<Settings>("./gen/schemas/settings.schema.json");
    write_schema::<Vec<AppConfig>>("./gen/schemas/settings_by_app.schema.json");

    write_schema::<Placeholder>("./gen/schemas/toolbar_items.schema.json");
    write_schema::<WegItems>("./gen/schemas/weg_items.schema.json");

    write_schema::<Theme>("./gen/schemas/theme.schema.json");
    write_schema::<Plugin>("./gen/schemas/plugin.schema.json");
    write_schema::<Widget>("./gen/schemas/widget.schema.json");
    write_schema::<IconPack>("./gen/schemas/icon_pack.schema.json");

    handlers::SeelenEvent::generate_ts_file("./src/handlers/events.ts");
    handlers::SeelenCommand::generate_ts_file("./src/handlers/commands.ts");
}
