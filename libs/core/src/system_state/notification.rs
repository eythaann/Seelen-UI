// All this structs/interfaces are taken from https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/schema-root

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct AppNotification {
    pub id: u32,
    pub app_umid: String,
    pub app_name: String,
    pub app_description: String,
    pub date: i64,
    pub content: Toast,
}

/// Base toast element, which contains at least a single visual element
#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct Toast {
    pub header: Option<ToastHeader>,
    pub visual: ToastVisual,
    pub actions: Option<ToastActions>,
    #[serde(rename = "@launch")]
    pub launch: Option<String>,
    #[serde(rename = "@activationType")]
    pub activation_type: ToastActionActivationType,
    #[serde(rename = "@duration")]
    pub duration: ToastDuration,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastDuration {
    #[default]
    Short,
    Long,
    #[serde(other)]
    Unknown,
}

/// Specifies a custom header that groups multiple notifications together within Action Center.
///
/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-header
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ToastHeader {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@arguments")]
    pub arguments: String,
    #[serde(default, rename = "@activationType")]
    pub activation_type: ToastActionActivationType,
}

/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-visual
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct ToastVisual {
    pub binding: ToastBinding,
    #[serde(rename = "@baseUri")]
    pub base_uri: String,
    #[serde(rename = "@lang")]
    pub lang: String,
    #[serde(rename = "@version")]
    pub version: u32,
    #[serde(rename = "@addImageQuery")]
    pub add_image_query: bool,
}

impl Default for ToastVisual {
    fn default() -> Self {
        ToastVisual {
            binding: Default::default(),
            base_uri: "ms-appx:///".to_owned(),
            lang: "none".to_owned(),
            version: 1,
            add_image_query: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct ToastBinding {
    #[serde(rename = "@template")]
    pub template: ToastTemplateType,
    #[serde(rename = "$value")]
    pub children: Vec<ToastBindingChild>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastTemplateType {
    ToastImageAndText01,
    ToastImageAndText02,
    ToastImageAndText03,
    ToastImageAndText04,
    ToastText01,
    ToastText02,
    ToastText03,
    ToastText04,
    #[default]
    ToastGeneric,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ToastBindingChild {
    Text(ToastText),
    Image(ToastImage),
    Group(ToastGroup),
    Progress(ToastProgress),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct ToastText {
    #[serde(rename = "@id")]
    pub id: Option<u32>,
    #[serde(rename = "$value")]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ToastImage {
    #[serde(rename = "@id")]
    pub id: Option<u32>,
    #[serde(rename = "@src")]
    pub src: String,
    #[serde(rename = "@alt")]
    pub alt: Option<String>,
    #[serde(default, rename = "@addImageQuery")]
    pub add_image_query: bool,
    #[serde(rename = "@placement")]
    pub placement: Option<ToastImagePlacement>,
    #[serde(rename = "@hint-crop")]
    pub hint_crop: Option<ToastImageCropType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastImageCropType {
    #[serde(alias = "circle")]
    Circle,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub enum ToastImagePlacement {
    #[serde(alias = "appLogoOverride")]
    AppLogoOverride,
    #[serde(alias = "hero")]
    Hero,
    #[serde(other)]
    Unknown,
}

/// Semantically identifies that the content in the group must either be displayed as a whole,
/// or not displayed if it cannot fit. Groups also allow creating multiple columns.
///
/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-group
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ToastGroup {
    pub subgroup: Vec<ToastSubGroup>,
}

/// Specifies vertical columns that can contain text and images.
///
/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-subgroup
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct ToastSubGroup {
    #[serde(rename = "$value")]
    pub children: Vec<ToastSubGroupChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ToastSubGroupChild {
    Text(ToastText),
    Image(ToastImage),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ToastProgress {
    #[serde(rename = "@title")]
    pub title: Option<String>,
    #[serde(rename = "@status")]
    pub status: String,
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(rename = "@valueStringOverride")]
    pub value_string_override: Option<String>,
}

/// Container element for declaring up to five inputs and up to five button actions for the toast notification.
///
/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-actions
#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct ToastActions {
    #[serde(rename = "$value")]
    pub children: Vec<ToastActionsChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ToastActionsChild {
    Input(ToastInput),
    Action(ToastAction),
}

/// Specifies an input, either text box or selection menu, shown in a toast notification.
///
/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-input
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ToastInput {
    /// The ID associated with the input
    #[serde(rename = "@id")]
    pub id: String,
    /// The type of input.
    #[serde(rename = "@type")]
    pub r#type: ToastInputType,
    /// The placeholder displayed for text input.
    #[serde(rename = "@placeHolderContent")]
    pub placeholder: Option<String>,
    /// Text displayed as a label for the input.
    #[serde(rename = "@title")]
    pub title: Option<String>,
    /// Options for the input if it is of type selection.
    #[serde(default)]
    pub selection: Vec<ToastInputSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastInputType {
    #[serde(alias = "text")]
    Text,
    #[serde(alias = "selection")]
    Selection,
    #[serde(other)]
    Unknown,
}

/// Specifies the id and text of a selection item.
///
/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-selection
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ToastInputSelection {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@content")]
    pub content: String,
}

/// Specifies a button shown in a toast.
///
/// https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-action
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ToastAction {
    #[serde(rename = "@content")]
    pub content: String,
    #[serde(rename = "@arguments")]
    pub arguments: String,
    #[serde(default, rename = "@activationType")]
    pub activation_type: ToastActionActivationType,
    #[serde(default, rename = "@afterActivationBehavior")]
    pub after_activation_behavior: ToastActionAfterActivationBehavior,
    /// if set to "contextMenu" then the action will be added to_string the context menu intead of the toast
    #[serde(rename = "@placement")]
    pub placement: Option<ToastActionPlacement>,
    /// this is used as button icon
    #[serde(rename = "@imageUri")]
    pub image_uri: Option<String>,
    #[serde(rename = "@hint-inputid")]
    pub hint_inputid: Option<String>,
    #[serde(rename = "@hint-buttonStyle")]
    pub hint_button_style: Option<ToastActionButtonStyle>,
    /// button tooltip
    #[serde(rename = "@hint-toolTip")]
    pub hint_tooltip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastActionButtonStyle {
    #[serde(alias = "success")]
    Sucess,
    #[serde(alias = "critical")]
    Critical,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastActionAfterActivationBehavior {
    #[default]
    #[serde(alias = "default")]
    Default,
    #[serde(alias = "pendingUpdate")]
    PendingUpdate,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastActionActivationType {
    #[default]
    #[serde(alias = "foreground")]
    Foreground,
    #[serde(alias = "background")]
    Background,
    #[serde(alias = "protocol")]
    Protocol,
    #[serde(alias = "system")]
    System,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum ToastActionPlacement {
    #[serde(alias = "contextMenu")]
    ContextMenu,
    #[serde(other)]
    Unknown,
}
