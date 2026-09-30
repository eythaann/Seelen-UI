use std::{thread, time::Duration};

use image::RgbaImage;
use windows::{
    Graphics::{
        Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem},
        DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    },
    Win32::{
        Foundation::{HMODULE, HWND, RPC_E_CHANGED_MODE},
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP},
            Direct3D11::{
                D3D11CreateDevice, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
                ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
            },
            Dxgi::IDXGIDevice,
        },
        System::WinRT::{
            Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess},
            Graphics::Capture::IGraphicsCaptureItemInterop,
            RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED,
        },
    },
    core::{factory, Interface},
};

use crate::error::Result;

/// Captures a window through Windows Graphics Capture.
///
/// This is the fallback for GPU-composited windows that do not expose a
/// redirection bitmap to PrintWindow, such as Electron and custom renderers.
pub fn capture_window(hwnd: isize) -> Result<RgbaImage> {
    let ro_initialized = match unsafe { RoInitialize(RO_INIT_MULTITHREADED) } {
        Ok(()) => true,
        Err(error) if error.code() == RPC_E_CHANGED_MODE => false,
        Err(error) => return Err(error.into()),
    };

    let result = capture_window_inner(hwnd);

    if ro_initialized {
        unsafe { RoUninitialize() };
    }

    result
}

fn capture_window_inner(hwnd: isize) -> Result<RgbaImage> {
    let (device, context) = create_device()?;
    let dxgi_device: IDXGIDevice = device.cast()?;
    let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)? };
    let graphics_device: IDirect3DDevice = inspectable.cast()?;

    let interop: IGraphicsCaptureItemInterop = factory::<GraphicsCaptureItem, _>()?;
    let item: GraphicsCaptureItem = unsafe { interop.CreateForWindow(HWND(hwnd as _))? };
    let size = item.Size()?;
    if size.Width <= 0 || size.Height <= 0 {
        return Err("Graphics capture returned an invalid window size".into());
    }

    let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &graphics_device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        size,
    )?;
    let session = frame_pool.CreateCaptureSession(&item)?;
    let _ = session.SetIsBorderRequired(false);
    session.StartCapture()?;

    let mut frame = None;
    for _ in 0..10 {
        if let Ok(next_frame) = frame_pool.TryGetNextFrame() {
            frame = Some(next_frame);
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let frame = frame.ok_or("Graphics capture did not produce a frame")?;

    let surface = frame.Surface()?;
    let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
    let source: ID3D11Texture2D = unsafe { access.GetInterface()? };

    let mut source_desc = D3D11_TEXTURE2D_DESC::default();
    unsafe { source.GetDesc(&mut source_desc) };

    let staging_desc = D3D11_TEXTURE2D_DESC {
        Width: source_desc.Width,
        Height: source_desc.Height,
        MipLevels: 1,
        ArraySize: 1,
        Format: source_desc.Format,
        SampleDesc: source_desc.SampleDesc,
        Usage: D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
        MiscFlags: 0,
    };
    let mut staging: Option<ID3D11Texture2D> = None;
    unsafe {
        device.CreateTexture2D(
            &staging_desc,
            None,
            Some(&mut staging as *mut Option<ID3D11Texture2D>),
        )?;
    }
    let staging = staging.ok_or("Failed to create staging texture")?;

    unsafe {
        context.CopyResource(&staging, &source);
    }

    let mut mapped = Default::default();
    unsafe {
        context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;
    }

    let width = source_desc.Width as usize;
    let height = source_desc.Height as usize;
    let row_size = width.checked_mul(4).ok_or("Graphics capture row is too large")?;
    let total_size = row_size
        .checked_mul(height)
        .ok_or("Graphics capture image is too large")?;
    let mut pixels = vec![0; total_size];

    if mapped.pData.is_null() {
        unsafe { context.Unmap(&staging, 0) };
        return Err("Graphics capture returned an empty frame".into());
    }

    for row in 0..height {
        let source_row = unsafe {
            std::slice::from_raw_parts(
                (mapped.pData as *const u8).add(row * mapped.RowPitch as usize),
                row_size,
            )
        };
        let target_row = &mut pixels[row * row_size..(row + 1) * row_size];
        for (source_pixel, target_pixel) in source_row
            .chunks_exact(4)
            .zip(target_row.chunks_exact_mut(4))
        {
            target_pixel[0] = source_pixel[2];
            target_pixel[1] = source_pixel[1];
            target_pixel[2] = source_pixel[0];
            target_pixel[3] = source_pixel[3];
        }
    }

    unsafe { context.Unmap(&staging, 0) };
    RgbaImage::from_raw(source_desc.Width, source_desc.Height, pixels)
        .ok_or_else(|| "Failed to create image from graphics capture frame".into())
}

fn create_device() -> Result<(ID3D11Device, ID3D11DeviceContext)> {
    for driver_type in [D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP] {
        let mut device = None;
        let mut context = None;
        let result = unsafe {
            D3D11CreateDevice(
                None::<&windows::Win32::Graphics::Dxgi::IDXGIAdapter>,
                driver_type,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device as *mut Option<ID3D11Device>),
                None,
                Some(&mut context as *mut Option<ID3D11DeviceContext>),
            )
        };

        if result.is_ok() {
            if let (Some(device), Some(context)) = (device, context) {
                return Ok((device, context));
            }
        }
    }

    Err("Failed to create a Direct3D 11 device for graphics capture".into())
}
