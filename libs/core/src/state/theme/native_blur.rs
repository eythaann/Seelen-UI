//! Experimental, per-WebView native background regions. Coordinates are CSS pixels.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct NativeBlurShape {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Elliptical corner radii, clockwise from top-left, normalized by CSS rules.
    pub radii: [[f32; 2]; 4],
    /// Optional convex outline in viewport CSS coordinates, for opted-in hinge projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(optional))]
    pub polygon: Option<Vec<[f32; 2]>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct NativeBlurRegion {
    pub id: u32,
    /// Element border box first, followed by ancestor padding-box clips.
    pub clips: Vec<NativeBlurShape>,
    pub amount: f32,
    pub opacity: f32,
    #[serde(default)]
    pub opacity_animation: Option<NativeBlurOpacityAnimation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct NativeBlurOpacityAnimation {
    pub id: u32,
    pub started_at_ms: f64,
    pub duration_ms: f32,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct NativeBlurCloseAnimation {
    pub region_id: u32,
    pub scale: f32,
    pub outlines: Vec<Vec<[f32; 2]>>,
    pub opacity: NativeBlurOpacityAnimation,
}
