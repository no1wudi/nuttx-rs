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
use libc::{O_NONBLOCK, O_RDONLY, c_int, c_void, open, read};

use crate::bindings::{
    TOUCH_DOWN, TOUCH_GESTURE_VALID, TOUCH_ID_VALID, TOUCH_MOVE, TOUCH_POS_VALID,
    TOUCH_PRESSURE_VALID, TOUCH_SIZE_VALID, TOUCH_UP, touch_point_s, touch_sample_s,
};

/// Represents a single touch point with position, size, pressure and timing information
///
/// This is an alias for the C `touch_point_s` structure from NuttX's touchscreen.h.
/// Each touch point has a unique ID that persists from touch down through move events
/// until the touch is released.
pub type TouchPoint = touch_point_s;

/// Contains a set of touch points from a single touch event
///
/// This matches the C `touch_sample_s` structure from NuttX's touchscreen.h.
/// The actual number of touch points is indicated by `npoints`, with the
/// points stored in the `point` array.
pub type TouchSample = touch_sample_s;

/// Represents an open connection to a touchscreen input device
///
/// Provides methods to read touch events and query touch state.
/// The device is opened in non-blocking mode by default.
pub struct TouchScreen {
    fd: c_int,
}

impl Default for TouchPoint {
    /// Creates a new TouchPoint with default/zero values
    fn default() -> Self {
        Self {
            id: 0,
            flags: 0,
            x: 0,
            y: 0,
            h: 0,
            w: 0,
            gesture: 0,
            pressure: 0,
            timestamp: 0,
        }
    }
}

impl TouchPoint {
    /// Checks if the touch point has valid position data
    ///
    /// # Returns
    /// true if the TOUCH_POS_VALID flag is set, indicating the x/y coordinates
    /// are valid
    pub fn is_pos_valid(&self) -> bool {
        self.flags & (TOUCH_POS_VALID as u8) != 0
    }

    /// Checks if this touch point represents a new touch down event
    ///
    /// # Returns
    /// true if the TOUCH_DOWN flag is set, indicating a new touch contact
    pub fn is_touch_down(&self) -> bool {
        self.flags & (TOUCH_DOWN as u8) != 0
    }

    /// Checks if this touch point represents a movement event
    ///
    /// # Returns
    /// true if the TOUCH_MOVE flag is set, indicating movement with previously reported contact
    pub fn is_touch_move(&self) -> bool {
        self.flags & (TOUCH_MOVE as u8) != 0
    }

    /// Checks if this touch point represents a touch release event
    ///
    /// # Returns
    /// true if the TOUCH_UP flag is set, indicating the touch contact was lost
    pub fn is_touch_up(&self) -> bool {
        self.flags & (TOUCH_UP as u8) != 0
    }

    /// Checks if the touch point ID is valid
    ///
    /// # Returns
    /// true if the TOUCH_ID_VALID flag is set, indicating the touch ID is certain
    pub fn is_id_valid(&self) -> bool {
        self.flags & (TOUCH_ID_VALID as u8) != 0
    }

    /// Checks if the touch point pressure data is valid
    ///
    /// # Returns
    /// true if the TOUCH_PRESSURE_VALID flag is set, indicating the pressure value is valid
    pub fn is_pressure_valid(&self) -> bool {
        self.flags & (TOUCH_PRESSURE_VALID as u8) != 0
    }

    /// Checks if the touch point size data is valid
    ///
    /// # Returns
    /// true if the TOUCH_SIZE_VALID flag is set, indicating the width/height values are valid
    pub fn is_size_valid(&self) -> bool {
        self.flags & (TOUCH_SIZE_VALID as u8) != 0
    }

    /// Checks if the touch point gesture data is valid
    ///
    /// # Returns
    /// true if the TOUCH_GESTURE_VALID flag is set, indicating the gesture value is valid
    pub fn is_gesture_valid(&self) -> bool {
        self.flags & (TOUCH_GESTURE_VALID as u8) != 0
    }
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
        let mut sample: TouchSample = unsafe { core::mem::zeroed() };

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
