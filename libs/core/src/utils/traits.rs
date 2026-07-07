#[cfg(feature = "salvo")]
pub trait SalvoBound: salvo::oapi::ToSchema + 'static {}

#[cfg(feature = "salvo")]
impl<T: salvo::oapi::ToSchema + 'static> SalvoBound for T {}

#[cfg(not(feature = "salvo"))]
pub trait SalvoBound {}

#[cfg(not(feature = "salvo"))]
impl<T> SalvoBound for T {}
