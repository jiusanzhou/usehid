//! Screenshot capture utilities
//!
//! Cross-platform screenshot capture returning PNG bytes.

use crate::error::{Error, Result};

/// Capture a full screenshot of the primary screen, returns PNG bytes.
pub fn screenshot() -> Result<Vec<u8>> {
    platform::capture_screen()
}

/// Capture a region of the screen, returns PNG bytes.
pub fn screenshot_region(x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>> {
    platform::capture_region(x, y, width, height)
}

// =============================================================================
// macOS: CoreGraphics + ImageIO
// =============================================================================
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::ffi::c_void;
    use std::ptr;

    // CoreGraphics types
    type CGImageRef = *mut c_void;
    type CFDataRef = *mut c_void;
    type CFMutableDataRef = *mut c_void;
    type CFStringRef = *const c_void;
    type CGImageDestinationRef = *mut c_void;
    type CFTypeRef = *const c_void;
    type CFAllocatorRef = *const c_void;
    type CFIndex = isize;

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;
    const K_CG_NULL_WINDOW_ID: u32 = 0;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGWindowListCreateImage(
            screenBounds: CGRect,
            listOption: u32,
            windowID: u32,
            imageOption: u32,
        ) -> CGImageRef;
        static CGRectInfinite: CGRect;
    }

    #[link(name = "ImageIO", kind = "framework")]
    extern "C" {
        fn CGImageDestinationCreateWithData(
            data: CFMutableDataRef,
            type_: CFStringRef,
            count: usize,
            options: CFTypeRef,
        ) -> CGImageDestinationRef;
        fn CGImageDestinationAddImage(
            dest: CGImageDestinationRef,
            image: CGImageRef,
            properties: CFTypeRef,
        );
        fn CGImageDestinationFinalize(dest: CGImageDestinationRef) -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDataCreateMutable(allocator: CFAllocatorRef, capacity: CFIndex) -> CFMutableDataRef;
        fn CFDataGetBytePtr(data: CFDataRef) -> *const u8;
        fn CFDataGetLength(data: CFDataRef) -> CFIndex;
        fn CFRelease(cf: CFTypeRef);

        static kCFAllocatorDefault: CFAllocatorRef;
    }

    // kUTTypePNG constant - we construct it via CFSTR
    extern "C" {
        fn CFStringCreateWithCString(
            alloc: CFAllocatorRef,
            c_str: *const i8,
            encoding: u32,
        ) -> CFStringRef;
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

    fn png_uti_string() -> CFStringRef {
        unsafe {
            CFStringCreateWithCString(
                kCFAllocatorDefault,
                b"public.png\0".as_ptr() as *const i8,
                K_CF_STRING_ENCODING_UTF8,
            )
        }
    }

    fn cg_image_to_png(image: CGImageRef) -> Result<Vec<u8>> {
        unsafe {
            let data = CFDataCreateMutable(kCFAllocatorDefault, 0);
            if data.is_null() {
                CGRelease(image);
                return Err(Error::ScreenshotFailed("Failed to create CFMutableData".into()));
            }

            let png_type = png_uti_string();
            let dest = CGImageDestinationCreateWithData(data, png_type, 1, ptr::null());
            CFRelease(png_type as CFTypeRef);

            if dest.is_null() {
                CFRelease(data as CFTypeRef);
                CGRelease(image);
                return Err(Error::ScreenshotFailed("Failed to create image destination".into()));
            }

            CGImageDestinationAddImage(dest, image, ptr::null());
            let ok = CGImageDestinationFinalize(dest);
            CFRelease(dest as CFTypeRef);
            CGRelease(image);

            if !ok {
                CFRelease(data as CFTypeRef);
                return Err(Error::ScreenshotFailed("Failed to finalize PNG".into()));
            }

            let ptr = CFDataGetBytePtr(data as CFDataRef);
            let len = CFDataGetLength(data as CFDataRef) as usize;
            let bytes = std::slice::from_raw_parts(ptr, len).to_vec();
            CFRelease(data as CFTypeRef);

            Ok(bytes)
        }
    }

    #[allow(non_snake_case)]
    unsafe fn CGRelease(image: CGImageRef) {
        if !image.is_null() {
            CFRelease(image as CFTypeRef);
        }
    }

    pub fn capture_screen() -> Result<Vec<u8>> {
        unsafe {
            let rect = CGRectInfinite;
            let image = CGWindowListCreateImage(
                rect,
                K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
                K_CG_NULL_WINDOW_ID,
                0, // kCGWindowImageDefault
            );
            if image.is_null() {
                return Err(Error::ScreenshotFailed("CGWindowListCreateImage returned null".into()));
            }
            cg_image_to_png(image)
        }
    }

    pub fn capture_region(x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>> {
        unsafe {
            let rect = CGRect {
                origin: CGPoint {
                    x: x as f64,
                    y: y as f64,
                },
                size: CGSize {
                    width: width as f64,
                    height: height as f64,
                },
            };
            let image = CGWindowListCreateImage(
                rect,
                K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
                K_CG_NULL_WINDOW_ID,
                0,
            );
            if image.is_null() {
                return Err(Error::ScreenshotFailed("CGWindowListCreateImage returned null for region".into()));
            }
            cg_image_to_png(image)
        }
    }
}

// =============================================================================
// Linux: X11 + png crate
// =============================================================================
#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::process::Command;

    pub fn capture_screen() -> Result<Vec<u8>> {
        // Use import command (ImageMagick) for simplicity; fallback to xwd + convert
        let output = Command::new("import")
            .args(["-window", "root", "png:-"])
            .output()
            .map_err(|e| Error::ScreenshotFailed(format!("import command failed: {}. Install ImageMagick.", e)))?;

        if !output.status.success() {
            return Err(Error::ScreenshotFailed(
                format!("import failed: {}", String::from_utf8_lossy(&output.stderr)),
            ));
        }

        Ok(output.stdout)
    }

    pub fn capture_region(x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>> {
        let geometry = format!("{}x{}+{}+{}", width, height, x, y);
        let output = Command::new("import")
            .args(["-window", "root", "-crop", &geometry, "png:-"])
            .output()
            .map_err(|e| Error::ScreenshotFailed(format!("import command failed: {}. Install ImageMagick.", e)))?;

        if !output.status.success() {
            return Err(Error::ScreenshotFailed(
                format!("import failed: {}", String::from_utf8_lossy(&output.stderr)),
            ));
        }

        Ok(output.stdout)
    }
}

// =============================================================================
// Windows: Win32 GDI + png crate
// =============================================================================
#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;

    pub fn capture_screen() -> Result<Vec<u8>> {
        unsafe {
            let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
            if hdc_screen.is_invalid() {
                return Err(Error::ScreenshotFailed("GetDC(NULL) failed".into()));
            }

            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            capture_dc(hdc_screen, 0, 0, width as u32, height as u32)?;
            let result = capture_dc(hdc_screen, 0, 0, width as u32, height as u32);
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
            result
        }
    }

    pub fn capture_region(x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>> {
        unsafe {
            let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
            if hdc_screen.is_invalid() {
                return Err(Error::ScreenshotFailed("GetDC(NULL) failed".into()));
            }

            let result = capture_dc(hdc_screen, x, y, width, height);
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
            result
        }
    }

    unsafe fn capture_dc(hdc: HDC, x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>> {
        let hdc_mem = CreateCompatibleDC(hdc);
        let hbm = CreateCompatibleBitmap(hdc, width as i32, height as i32);
        let old = SelectObject(hdc_mem, hbm);

        BitBlt(hdc_mem, 0, 0, width as i32, height as i32, hdc, x, y, SRCCOPY)
            .map_err(|e| Error::ScreenshotFailed(format!("BitBlt failed: {}", e)))?;

        // Extract pixel data
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels = vec![0u8; (width * height * 4) as usize];
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old);
        DeleteObject(hbm);
        DeleteDC(hdc_mem);

        // Encode as PNG using png crate
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .map_err(|e| Error::ScreenshotFailed(format!("PNG encode error: {}", e)))?;

            // Convert BGRA to RGBA
            for chunk in pixels.chunks_exact_mut(4) {
                chunk.swap(0, 2); // B <-> R
            }

            writer
                .write_image_data(&pixels)
                .map_err(|e| Error::ScreenshotFailed(format!("PNG write error: {}", e)))?;
        }

        Ok(png_data)
    }
}

// =============================================================================
// Unsupported platforms
// =============================================================================
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use super::*;

    pub fn capture_screen() -> Result<Vec<u8>> {
        Err(Error::PlatformNotSupported("screenshot".into()))
    }

    pub fn capture_region(_x: i32, _y: i32, _width: u32, _height: u32) -> Result<Vec<u8>> {
        Err(Error::PlatformNotSupported("screenshot_region".into()))
    }
}
