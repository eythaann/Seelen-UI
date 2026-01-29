#![allow(dead_code, non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WmiMonitorBrightness {
    pub active: bool,
    pub current_brightness: u8,
    pub instance_name: String,
    pub level: Vec<u8>,
    pub levels: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WmiMonitorBrightnessEvent {
    pub active: bool,
    pub brightness: u8,
    pub instance_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WmiMonitorBrightnessMethods {
    #[serde(rename = "__Path")]
    pub __path: String,
    pub active: bool,
    pub instance_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct WmiSetBrightnessPayload {
    pub timeout: u32,
    pub brightness: u8,
}
