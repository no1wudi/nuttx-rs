//! Framebuffer interface bindings for NuttX
//!
//! This module provides Rust bindings for the NuttX framebuffer driver interface.
//! It allows interacting with framebuffer devices to get display information,
//! manage framebuffer memory, and update display regions.
//!
//! The implementation matches the NuttX framebuffer interface defined in
//! `nuttx/include/nuttx/video/fb.h`.
//!
//! # Examples
//!
//! ```no_run
//! use core::ffi::CStr;
//! use nuttx::video::fb::{FrameBuffer, Format};
//!
//! let fb = FrameBuffer::new(CStr::from_bytes_with_nul(b"/dev/fb0\0").unwrap()).unwrap();
//! let info = fb.get_video_info().unwrap();
//! assert_eq!(info.fmt, Format::RGB565 as u8);
//! ```

use core::ffi::{CStr, c_void};

/// Color format definitions
///
/// Matches C's FB_FMT_* constants
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// BPP=16, R=5, G=6, B=5
    RGB565 = 11,
}

/// Coordinate type used in framebuffer structures
///
/// Matches C's `fb_coord_t` which is a uint16_t
pub type Coord = u16;

/// IOCTL command to get video information
///
/// Matches C's FBIOGET_VIDEOINFO
pub const FBIOGET_VIDEOINFO: i32 = 0x2801;

/// IOCTL command to get plane information
///
/// Matches C's FBIOGET_PLANEINFO
pub const FBIOGET_PLANEINFO: i32 = 0x2802;

/// IOCTL command to update a rectangular region in the framebuffer
///
/// Matches C's FBIO_UPDATE
pub const FBIO_UPDATE: i32 = 0x2807;

/// Video controller information structure
///
/// Matches C's `fb_videoinfo_s`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VideoInfo {
    /// Color format, see Format enum
    pub fmt: u8,
    /// Horizontal resolution in pixel columns
    pub xres: Coord,
    /// Vertical resolution in pixel rows
    pub yres: Coord,
    /// Number of color planes supported
    pub nplanes: u8,
    /// Number of overlays supported (if fb_overlay feature enabled)
    #[cfg(feature = "fb_overlay")]
    pub noverlays: u8,
    /// Module information filled by vendor (if fb_moduleinfo feature enabled)
    #[cfg(feature = "fb_moduleinfo")]
    pub moduleinfo: [u8; 128],
}

/// Plane information structure
///
/// Matches C's `fb_planeinfo_s`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaneInfo {
    /// Start of frame buffer memory
    pub fbmem: usize,
    /// Length of frame buffer memory in bytes
    pub fblen: usize,
    /// Length of a line in bytes
    pub stride: Coord,
    /// Display number
    pub display: u8,
    /// Bits per pixel
    pub bpp: u8,
    /// Virtual Horizontal resolution in pixel columns
    pub xres_virtual: u32,
    /// Virtual Vertical resolution in pixel rows
    pub yres_virtual: u32,
    /// Offset from virtual to visible resolution
    pub xoffset: u32,
    /// Offset from virtual to visible resolution
    pub yoffset: u32,
}

/// Area structure describing a rectangular region
///
/// Matches C's `fb_area_s`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Area {
    /// x-offset of the area
    pub x: Coord,
    /// y-offset of the area
    pub y: Coord,
    /// Width of the area
    pub w: Coord,
    /// Height of the area
    pub h: Coord,
}

/// Result type for framebuffer operations
pub type FrameBufferResult<T> = Result<T, i32>;

/// FrameBuffer structure wrapping the framebuffer functionality
#[derive(Debug)]
pub struct FrameBuffer {
    fd: i32,
}

impl FrameBuffer {
    /// Open the framebuffer device
    pub fn new(path: &CStr) -> FrameBufferResult<Self> {
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            return Err(fd);
        }
        Ok(Self { fd })
    }

    /// Get video information from the framebuffer device
    ///
    /// # Returns
    /// `VideoInfo` structure containing the framebuffer's video information
    ///
    /// # Errors
    /// Returns `FrameBufferError::PlaneInfoFailed` if the ioctl fails
    pub fn get_video_info(&self) -> FrameBufferResult<VideoInfo> {
        let mut info = VideoInfo::default();

        // SAFETY: We're passing valid pointers to the ioctl
        let result = unsafe {
            libc::ioctl(
                self.fd,
                FBIOGET_VIDEOINFO.try_into().unwrap(),
                &mut info as *mut VideoInfo as *mut c_void,
            )
        };

        if result < 0 { Err(result) } else { Ok(info) }
    }

    /// Get plane information from the framebuffer device
    ///
    /// # Returns
    /// `PlaneInfo` structure containing the framebuffer's plane information
    ///
    /// # Errors
    /// Returns a libc error code if the ioctl fails
    pub fn get_plane_info(&self) -> FrameBufferResult<PlaneInfo> {
        let mut info = PlaneInfo::default();

        // SAFETY: We're passing valid pointers to the ioctl
        let result = unsafe {
            libc::ioctl(
                self.fd,
                FBIOGET_PLANEINFO.try_into().unwrap(),
                &mut info as *mut PlaneInfo as *mut c_void,
            )
        };

        if result < 0 { Err(result) } else { Ok(info) }
    }

    /// Update a rectangular region in the framebuffer
    ///
    /// # Arguments
    /// * `area` - The rectangular region to update
    ///
    /// # Errors
    /// Returns a libc error code if the ioctl fails
    #[cfg(feature = "fb_update")]
    pub fn update_area(&self, area: &Area) -> FrameBufferResult<()> {
        // SAFETY: We're passing valid pointers to the ioctl
        let result = unsafe {
            libc::ioctl(
                self.fd,
                FBIO_UPDATE as libc::c_ulong,
                area as *const Area as *mut c_void,
            )
        };

        if result < 0 {
            Err(result as libc::c_int)
        } else {
            Ok(())
        }
    }

    #[cfg(not(feature = "fb_update"))]
    pub fn update_area(&self, _area: &Area) -> FrameBufferResult<()> {
        Ok(())
    }
}

impl Drop for FrameBuffer {
    /// Automatically closes the framebuffer device when the FrameBuffer instance goes out of scope
    ///
    /// This ensures that system resources are properly released even if the FrameBuffer
    /// instance is not explicitly closed. The underlying file descriptor is closed
    /// using the libc::close() function.
    ///
    /// # Safety
    /// This function is marked unsafe because it calls into C code through libc::close().
    /// The file descriptor is guaranteed to be valid as it's managed by the FrameBuffer
    /// struct and only set during successful initialization.
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
