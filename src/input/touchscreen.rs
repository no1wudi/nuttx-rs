//! Touchscreen input device interface
//!
//! This module provides Rust bindings for the NuttX touchscreen driver interface.
//! It allows reading touch events and gestures from a touchscreen device.
//!
//! The implementation matches the NuttX touchscreen interface defined in
//! `nuttx/include/nuttx/input/touchscreen.h`.
//!

use core::ffi::CStr;
use core::mem::size_of;
use libc::{O_RDONLY, c_int, c_void, open, read, fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

/// Represents a single touch point with position, size, pressure and timing information
///
/// This matches the C `touch_point_s` structure from NuttX's touchscreen.h.
/// Each touch point has a unique ID that persists from touch down through move events
/// until the touch is released.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TouchPoint {
    /// Unique identifies contact; Same in all reports for the contact
    pub id: u8,
    /// See TOUCH_* definitions above
    pub flags: u8,
    /// X coordinate of the touch point (uncalibrated)
    pub x: i16,
    /// Y coordinate of the touch point (uncalibrated)
    pub y: i16,
    /// Height of touch point (uncalibrated)
    pub h: i16,
    /// Width of touch point (uncalibrated)
    pub w: i16,
    /// Gesture of touchscreen contact
    pub gesture: u16,
    /// Touch pressure
    pub pressure: u16,
    /// Touch event time stamp, in microseconds
    pub timestamp: u64,
}

/// Contains a set of touch points from a single touch event
///
/// This matches the C `touch_sample_s` structure from NuttX's touchscreen.h.
/// The actual number of touch points is indicated by `npoints`, with the
/// points stored in the `point` array.
#[repr(C)]
pub struct TouchSample {
    /// The number of touch points in point[]
    pub npoints: c_int,
    /// Actual dimension is npoints
    pub point: [TouchPoint; 1],
}

impl TouchPoint {
    /// Creates a new TouchPoint with default/zero values
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if the touch point has valid position data
    ///
    /// # Returns
    /// true if the TOUCH_POS_VALID flag is set, indicating the x/y coordinates
    /// are valid
    pub fn is_pos_valid(&self) -> bool {
        self.flags & TOUCH_POS_VALID != 0
    }

    /// Checks if this touch point represents a new touch down event
    ///
    /// # Returns
    /// true if the TOUCH_DOWN flag is set, indicating a new touch contact
    pub fn is_touch_down(&self) -> bool {
        self.flags & TOUCH_DOWN != 0
    }

    /// Checks if this touch point represents a movement event
    ///
    /// # Returns
    /// true if the TOUCH_MOVE flag is set, indicating movement with previously reported contact
    pub fn is_touch_move(&self) -> bool {
        self.flags & TOUCH_MOVE != 0
    }

    /// Checks if this touch point represents a touch release event
    ///
    /// # Returns
    /// true if the TOUCH_UP flag is set, indicating the touch contact was lost
    pub fn is_touch_up(&self) -> bool {
        self.flags & TOUCH_UP != 0
    }

    /// Checks if the touch point ID is valid
    ///
    /// # Returns
    /// true if the TOUCH_ID_VALID flag is set, indicating the touch ID is certain
    pub fn is_id_valid(&self) -> bool {
        self.flags & TOUCH_ID_VALID != 0
    }

    /// Checks if the touch point pressure data is valid
    ///
    /// # Returns
    /// true if the TOUCH_PRESSURE_VALID flag is set, indicating the pressure value is valid
    pub fn is_pressure_valid(&self) -> bool {
        self.flags & TOUCH_PRESSURE_VALID != 0
    }

    /// Checks if the touch point size data is valid
    ///
    /// # Returns
    /// true if the TOUCH_SIZE_VALID flag is set, indicating the width/height values are valid
    pub fn is_size_valid(&self) -> bool {
        self.flags & TOUCH_SIZE_VALID != 0
    }

    /// Checks if the touch point gesture data is valid
    ///
    /// # Returns
    /// true if the TOUCH_GESTURE_VALID flag is set, indicating the gesture value is valid
    pub fn is_gesture_valid(&self) -> bool {
        self.flags & TOUCH_GESTURE_VALID != 0
    }
}

/// A new touch contact is established
pub const TOUCH_DOWN: u8 = 1 << 0;
/// Movement occurred with previously reported contact
pub const TOUCH_MOVE: u8 = 1 << 1;
/// The touch contact was lost
pub const TOUCH_UP: u8 = 1 << 2;
/// Touch ID is certain
pub const TOUCH_ID_VALID: u8 = 1 << 3;
/// Hardware provided a valid X/Y position
pub const TOUCH_POS_VALID: u8 = 1 << 4;
/// Hardware provided a valid pressure
pub const TOUCH_PRESSURE_VALID: u8 = 1 << 5;
/// Hardware provided a valid H/W contact size
pub const TOUCH_SIZE_VALID: u8 = 1 << 6;
/// Hardware provided a valid gesture
pub const TOUCH_GESTURE_VALID: u8 = 1 << 7;

/// Double click gesture
pub const TOUCH_DOUBLE_CLICK: u16 = 0x00;
/// Slide up gesture
pub const TOUCH_SLIDE_UP: u16 = 0x01;
/// Slide down gesture
pub const TOUCH_SLIDE_DOWN: u16 = 0x02;
/// Slide left gesture
pub const TOUCH_SLIDE_LEFT: u16 = 0x03;
/// Slide right gesture
pub const TOUCH_SLIDE_RIGHT: u16 = 0x04;
/// Palm gesture
pub const TOUCH_PALM: u16 = 0x05;

/// Represents an open connection to a touchscreen input device
///
/// Provides methods to read touch events and query touch state.
/// The device is opened in non-blocking mode by default.
pub struct TouchScreen {
    fd: c_int,
}

impl TouchScreen {
    /// Opens a touchscreen device at the specified path
    ///
    /// # Arguments
    /// * `path` - Path to the touch device as a C string (e.g. "/dev/input0")
    ///
    /// # Returns
    /// - Ok(TouchScreen) on success
    /// - Err(i32) with error code if the device could not be opened
    pub fn open(path: &CStr) -> Result<Self, i32> {
        let fd = unsafe { open(path.as_ptr(), O_RDONLY | O_NONBLOCK) };
        if fd < 0 {
            return Err(fd);
        }

        Ok(TouchScreen { fd })
    }

    /// Reads a touch sample from the device
    ///
    /// This reads the next available touch event from the device. The device is opened
    /// in non-blocking mode by default, so if no touch data is available this will
    /// return immediately with a sample containing npoints = 0.
    ///
    /// # Returns
    /// - Ok(TouchSample) containing the touch data. The sample will have:
    ///   - npoints = 0 if no touch data is available
    ///   - npoints = 1 for single-touch devices
    ///   - npoints > 1 for multi-touch devices (if supported)
    /// - Err(i32) with the error code if the read operation failed
    ///
    /// # Errors
    /// Returns an error if:
    /// - The device is not properly opened
    /// - The read operation fails
    /// - The buffer is too small for the received data
    ///
    /// # Notes
    /// - The TouchSample structure uses a fixed-size array for touch points, but
    ///   multi-touch devices may report more points than can be stored. In this case,
    ///   only the first point will be available.
    /// - Check the flags field in each TouchPoint to determine if the data is valid
    pub fn read_sample(&mut self) -> Result<TouchSample, i32> {
        let mut sample = TouchSample {
            npoints: 0,
            point: [TouchPoint::default()],
        };

        let bytes_read = unsafe {
            read(
                self.fd,
                &mut sample as *mut _ as *mut c_void,
                size_of::<TouchSample>(),
            )
        };

        if bytes_read < 0 {
            return Err(bytes_read as i32);
        } else if bytes_read as usize != size_of::<TouchSample>() {
            return Err(-libc::EIO); // Input/output error for incomplete read
        }
        Ok(sample)
    }
}

impl Drop for TouchScreen {
    /// Automatically closes the touchscreen device when the TouchScreen instance goes out of scope
    ///
    /// This ensures that system resources are properly released even if the TouchScreen
    /// instance is not explicitly closed. The underlying file descriptor is closed
    /// using the libc::close() function.
    ///
    /// # Safety
    /// This function is marked unsafe because it calls into C code through libc::close().
    /// The file descriptor is guaranteed to be valid as it's managed by the TouchScreen
    /// struct and only set during successful initialization.
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
