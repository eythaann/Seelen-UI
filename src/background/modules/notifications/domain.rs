use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    pub id: u32,
    pub app_umid: String,
    pub app_name: String,
    pub app_description: String,
    pub body: Vec<String>,
    pub date: i64,
}
