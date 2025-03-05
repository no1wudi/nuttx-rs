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
//! use nuttx::video::fb::{FrameBuffer, FB_FMT_RGB16_565};
//!
//! let fb = FrameBuffer::new(CStr::from_bytes_with_nul(b"/dev/fb0\0").unwrap()).unwrap();
//! let info = fb.get_video_info().unwrap();
//! assert_eq!(info.fmt, FB_FMT_RGB16_565);
//! ```

use crate::bindings;
use core::ffi::{CStr, c_void};
use kconfig::kconfig;

// Re-export only RGB family of framebuffer format constants
pub use bindings::{
    FB_FMT_RGB4, FB_FMT_RGB8, FB_FMT_RGB8_222, FB_FMT_RGB8_332, FB_FMT_RGB12_444, FB_FMT_RGB16_555,
    FB_FMT_RGB16_565, FB_FMT_RGB24, FB_FMT_RGB32, FB_FMT_RGBA16, FB_FMT_RGBA32,
};

/// Coordinate type used in framebuffer structures
///
/// Matches C's `fb_coord_t` which is a uint16_t
pub type Coord = bindings::fb_coord_t;

/// Video controller information structure
///
/// Alias for C's `fb_videoinfo_s`
pub type VideoInfo = bindings::fb_videoinfo_s;

/// Plane information structure
///
/// Alias for C's `fb_planeinfo_s`
pub type PlaneInfo = bindings::fb_planeinfo_s;

/// Area structure describing a rectangular region
///
/// Alias for C's `fb_area_s`
pub type Area = bindings::fb_area_s;

/// IOCTL command to get video information
///
/// Matches C's FBIOGET_VIDEOINFO
const FBIOGET_VIDEOINFO: i32 = 0x2801;

/// IOCTL command to get plane information
///
/// Matches C's FBIOGET_PLANEINFO
const FBIOGET_PLANEINFO: i32 = 0x2802;

/// IOCTL command to update a rectangular region in the framebuffer
///
/// Matches C's FBIO_UPDATE
#[allow(dead_code)]
const FBIO_UPDATE: i32 = 0x2807;

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
        let mut info = unsafe { core::mem::zeroed::<VideoInfo>() };

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
        let mut info = unsafe { core::mem::zeroed::<PlaneInfo>() };

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
    #[kconfig(CONFIG_FB_UPDATE = "y")]
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

    #[kconfig(CONFIG_FB_UPDATE = "n")]
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
