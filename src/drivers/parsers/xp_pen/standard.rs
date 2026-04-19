use crate::drivers::TabletData;

pub fn parse(data: &[u8]) -> Option<TabletData> {
    if data.len() < 8 {
        return None;
    }

    // XP-Pen usually uses this format for G-series / Gen2
    let x = ((data[3] as u16) << 8) | (data[2] as u16);
    let y = ((data[5] as u16) << 8) | (data[4] as u16);
    let pressure = ((data[7] as u16) << 8) | (data[6] as u16);
    
    // Tilt (X at 8, Y at 9)
    let tilt_x = data.get(8).copied().unwrap_or(0) as i8;
    let tilt_y = data.get(9).copied().unwrap_or(0) as i8;

    // Buttons & Eraser (Byte 1)
    // Bit 1 = Button 0
    // Bit 2 = Button 1
    // Bit 3 = Eraser
    let buttons = (data[1] >> 1) & 0x03; // Correctly masks bits 1 and 2
    let eraser = (data[1] & 0x08) != 0;

    // Raw hex string for debugging
    let raw = data.iter()
        .take(14)
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    let status = match data[1] {
        0xA0 => "Hover",
        0xA1 => "Contact",
        0xC0 | 0x00 => "Out of Range",
        _ => "Active",
    }.to_string();
    
    let is_connected = status != "Out of Range";

    Some(TabletData {
        status,
        x,
        y,
        pressure,
        tilt_x,
        tilt_y,
        buttons,
        eraser,
        hover_distance: 0, // Not provided in this report
        raw_data: raw,
        is_connected,
        ..Default::default()
    })
}
