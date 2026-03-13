//! # Mathematics and Transforms
//!
//! This module provides the mathematical functions required to map raw
//! physical tablet coordinates into operating system screen coordinates.
//! It handles aspects like rotation, active area cropping, absolute-to-relative
//! conversion, and respecting aspect ratios.

/// Applies 2D rotation around a center point (usually 0.5, 0.5 in normalized space)
/// Or directly to continuous values.
pub fn rotate_point(
    x: f32,
    y: f32,
    center_x: f32,
    center_y: f32,
    rotation_degrees: f32,
) -> (f32, f32) {
    if rotation_degrees == 0.0 {
        return (x, y);
    }
    let rad = -rotation_degrees.to_radians();
    let (sin, cos) = rad.sin_cos();

    let cx = x - center_x;
    let cy = y - center_y;

    let rx = cx * cos - cy * sin + center_x;
    let ry = cx * sin + cy * cos + center_y;

    (rx, ry)
}

/// Normalizes physical tablet coordinates (millimeters) into [0.0, 1.0] UV space,
/// taking the user-defined active area and tablet rotation into account.
///
/// This is the first step of the **Absolute** mapping pipeline.
///
/// # Arguments
/// * `x_mm`, `y_mm` - Raw physical position from the tablet, converted to millimeters.
/// * `area_x`, `area_y` - The top-left offset of the mapped active area (mm).
/// * `area_w`, `area_h` - The width and height of the mapped active area (mm).
/// * `rotation` - Degrees to rotate the output space around its center.
///
/// # Returns
/// A tuple `(u, v)` representing the pen's position as a percentage of the active area.
/// Note: These values can briefly exceed `0.0` or `1.0` if the pen moves slightly outside
/// the mathematically defined active area before being clamped by the screen projection.
pub fn physical_to_normalized(
    x_mm: f32,
    y_mm: f32,
    area_x: f32,
    area_y: f32,
    area_w: f32,
    area_h: f32,
    rotation: f32,
) -> (f32, f32) {
    // 1. Normalize (Tablet Space in mm) - Center Based
    let mut u = (x_mm - area_x) / area_w + 0.5;
    let mut v = (y_mm - area_y) / area_h + 0.5;

    // 2. Apply Rotation
    if rotation != 0.0 {
        let (nu, nv) = rotate_point(u, v, 0.5, 0.5, rotation);
        u = nu;
        v = nv;
    }

    (u, v)
}

/// Projects normalized UV coordinates `[0.0, 1.0]` onto physical Screen Space pixels.
///
/// This is the second step of the **Absolute** mapping pipeline following normalization.
///
/// # Arguments
/// * `u`, `v` - Normalized percentage coordinates, clamped internally by this function.
/// * `target_x`, `target_y` - The top-left origin of the display map (e.g., `0,0` for primary).
/// * `target_w`, `target_h` - The resolution/dimensions of the target screen area.
///
/// # Returns
/// Sub-pixel precise `(x, y)` coordinates in OS screen space.
pub fn normalized_to_screen(
    u: f32,
    v: f32,
    target_x: f32,
    target_y: f32,
    target_w: f32,
    target_h: f32,
) -> (f32, f32) {
    let u_clamped = u.clamp(0.0, 1.0);
    let v_clamped = v.clamp(0.0, 1.0);

    let screen_x = target_x + u_clamped * target_w;
    let screen_y = target_y + v_clamped * target_h;

    (screen_x, screen_y)
}

/// Computes the delta movement for **Relative** (mouse-like) driver mode.
///
/// Instead of mapping absolute physical positions to absolute screen positions,
/// relative mode acts like a traditional mouse, moving the cursor by an offset
/// from its current location.
///
/// # Arguments
/// * `x_mm`, `y_mm` - The current physical location of the pen.
/// * `last_x_mm`, `last_y_mm` - The previous physical location of the pen.
/// * `rotation` - Rotational offset applied to the movement vector.
/// * `sens_x`, `sens_y` - Sensitivity multipliers (Pixels per Millimeter).
///
/// # Returns
/// A tuple `(dx_px, dy_px)` representing how many pixels the cursor should be
/// translated by this tick.
pub fn apply_relative_delta(
    x_mm: f32,
    y_mm: f32,
    last_x_mm: f32,
    last_y_mm: f32,
    rotation: f32,
    sens_x: f32,
    sens_y: f32,
) -> (f32, f32) {
    let mut dx_mm = x_mm - last_x_mm;
    let mut dy_mm = y_mm - last_y_mm;

    // Apply rotation to the movement vector
    if rotation != 0.0 {
        let rad = rotation.to_radians(); // Note: Relative mode rotation is typically positive towards the right.
        let (sin, cos) = rad.sin_cos();
        let rx = dx_mm * cos - dy_mm * sin;
        let ry = dx_mm * sin + dy_mm * cos;
        dx_mm = rx;
        dy_mm = ry;
    }

    // Apply sensitivity mapping
    let dx_px = dx_mm * sens_x;
    let dy_px = dy_mm * sens_y;

    (dx_px, dy_px)
}
