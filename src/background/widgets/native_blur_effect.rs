//! GPU backdrop effect used by the Widget renderer and native resource tests.

use std::cell::RefCell;

use crate::error::Result;
use windows::{
    Foundation::{IPropertyValue, PropertyValue},
    Graphics::Effects::{IGraphicsEffect, IGraphicsEffectSource},
    UI::Composition::{CompositionEffectBrush, CompositionEffectSourceParameter, Compositor},
    Win32::{
        Graphics::Direct2D::CLSID_D2D1GaussianBlur,
        System::WinRT::{
            CreateDispatcherQueueController, DQTAT_COM_STA, DQTYPE_THREAD_CURRENT,
            DispatcherQueueOptions,
            Graphics::Direct2D::{
                GRAPHICS_EFFECT_PROPERTY_MAPPING, GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT,
                IGraphicsEffectD2D1Interop, IGraphicsEffectD2D1Interop_Impl,
            },
        },
    },
    core::{HSTRING, Interface, PCWSTR, implement},
};

const SOURCE_NAME: &str = "backdrop";

thread_local! {
    static DISPATCHER: RefCell<Option<windows::System::DispatcherQueueController>> = const { RefCell::new(None) };
}

#[cfg(test)]
thread_local! { pub(super) static TEST_BRUSH_CREATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

pub(super) fn create_blur_brush(
    compositor: &Compositor,
    blur_amount: f32,
) -> Result<CompositionEffectBrush> {
    let source: IGraphicsEffectSource =
        CompositionEffectSourceParameter::Create(&HSTRING::from(SOURCE_NAME))?.cast()?;
    let effect: IGraphicsEffect = GaussianBlurEffect {
        source,
        blur_amount,
    }
    .into();
    let factory = probe(
        "CreateEffectFactory",
        compositor.CreateEffectFactory(&effect),
    )?;
    let brush = probe("CreateBrush", factory.CreateBrush())?;
    let backdrop = compositor.CreateBackdropBrush()?;
    probe(
        "SetSourceParameter",
        brush.SetSourceParameter(&HSTRING::from(SOURCE_NAME), &backdrop),
    )?;
    #[cfg(test)]
    TEST_BRUSH_CREATIONS.with(|count| count.set(count.get() + 1));
    Ok(brush)
}

pub(super) fn ensure_dispatcher() -> Result<()> {
    DISPATCHER.with_borrow_mut(|dispatcher| -> Result<()> {
        if dispatcher.is_none() && windows::System::DispatcherQueue::GetForCurrentThread().is_err()
        {
            let options = DispatcherQueueOptions {
                dwSize: std::mem::size_of::<DispatcherQueueOptions>() as u32,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_STA,
            };
            *dispatcher = Some(unsafe { CreateDispatcherQueueController(options)? });
        }
        Ok(())
    })
}

#[implement(IGraphicsEffect, IGraphicsEffectSource, IGraphicsEffectD2D1Interop)]
struct GaussianBlurEffect {
    source: IGraphicsEffectSource,
    blur_amount: f32,
}

impl windows::Graphics::Effects::IGraphicsEffectSource_Impl for GaussianBlurEffect_Impl {}

impl windows::Graphics::Effects::IGraphicsEffect_Impl for GaussianBlurEffect_Impl {
    fn Name(&self) -> windows::core::Result<HSTRING> {
        Ok(HSTRING::from("SeelenNativeBlur"))
    }

    fn SetName(&self, _name: &HSTRING) -> windows::core::Result<()> {
        Ok(())
    }
}

#[allow(non_snake_case)]
impl IGraphicsEffectD2D1Interop_Impl for GaussianBlurEffect_Impl {
    fn GetEffectId(&self) -> windows::core::Result<windows::core::GUID> {
        Ok(CLSID_D2D1GaussianBlur)
    }

    fn GetNamedPropertyMapping(
        &self,
        name: &PCWSTR,
        index: *mut u32,
        mapping: *mut GRAPHICS_EFFECT_PROPERTY_MAPPING,
    ) -> windows::core::Result<()> {
        if index.is_null() || mapping.is_null() {
            return Err(invalid_argument());
        }
        let property = match unsafe { name.to_string()? }.as_str() {
            "BlurAmount" => 0, // D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION
            "Optimization" => 1,
            "BorderMode" => 2,
            _ => return Err(invalid_argument()),
        };
        unsafe {
            *index = property;
            *mapping = GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT;
        }
        Ok(())
    }

    fn GetPropertyCount(&self) -> windows::core::Result<u32> {
        Ok(3)
    }

    fn GetProperty(&self, index: u32) -> windows::core::Result<IPropertyValue> {
        match index {
            0 => PropertyValue::CreateSingle(self.blur_amount)?.cast(),
            1 => PropertyValue::CreateUInt32(1)?.cast(), // Balanced optimization
            2 => PropertyValue::CreateUInt32(1)?.cast(), // Hard border: avoid transparent edge falloff.
            _ => Err(invalid_argument()),
        }
    }

    fn GetSource(&self, index: u32) -> windows::core::Result<IGraphicsEffectSource> {
        if index != 0 {
            return Err(invalid_argument());
        }
        Ok(self.source.clone())
    }

    fn GetSourceCount(&self) -> windows::core::Result<u32> {
        Ok(1)
    }
}

fn invalid_argument() -> windows::core::Error {
    windows::core::Error::from_hresult(windows::core::HRESULT(0x80070057u32 as i32))
}

fn probe<T>(stage: &str, result: windows::core::Result<T>) -> windows::core::Result<T> {
    result.map_err(|error| {
        log::warn!("Native blur probe {stage} failed: {error}");
        error
    })
}
