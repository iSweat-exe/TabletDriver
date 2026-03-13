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

/// Normalizes physical tablet coordinates (millimeters) into [0.0, 1.0] UV space
/// taking the active area and rotation into account.
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

/// Project normalized UV coordinates to Screen Space pixels limits.
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

/// Applies Relative translation, tracking the delta in mm, rotating it, and applying pixel/mm sensitivities.
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
