use windows::Win32::System::WinRT::EventRegistrationToken;

pub trait EventRegistrationTokenExt {
    /// some funtions need to return EventRegistrationToken, but instead return i64
    /// so this trait is to facilitate the conversion, and reduce the amount of refactor for
    /// upgrating to futures windows crate versions
    fn as_event_token(&self) -> EventRegistrationToken;
}

impl EventRegistrationTokenExt for i64 {
    fn as_event_token(&self) -> EventRegistrationToken {
        EventRegistrationToken { value: *self }
    }
}
