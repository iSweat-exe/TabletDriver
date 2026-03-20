use crate::drivers::parsers::ReportParser;
use crate::drivers::TabletData;

pub struct FallbackParser;

impl ReportParser for FallbackParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        // Universal/Fallback Parser
        // Attempts to parse using the most common structure found in many graphics tablets (Huion, Gaomon, Monoprice, etc)
        // Usually: [ReportID, Buttons/Status, X_LSB, X_MSB, Y_LSB, Y_MSB, P_LSB, P_MSB, ...]

        if data.len() < 8 {
            return None;
        }

        // Check if index 0 is Report ID or data.
        // If data[0] is constant and data[1] looks like buttons, it might be.
        // However, most HIDAPI reads on Windows include the ReportId at 0.

        // Let's assume standard layout at offset 2 for X
        let x = ((data[3] as u16) << 8) | (data[2] as u16);
        let y = ((data[5] as u16) << 8) | (data[4] as u16);
        let pressure = ((data[7] as u16) << 8) | (data[6] as u16);

        let raw = data
            .iter()
            .take(10)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        // Status Byte usually at index 1
        // 0xC0 = Range Out, 0xA0 = Hover, 0xA1 = Touch/Down
        // OR standard bitmasks
        let status_byte = data[1];

        // Strict Mode: Treat 0x00 and 0xC0 as Out of Range
        let status = if status_byte == 0xC0 || status_byte == 0x00 {
            "Out of Range".to_string()
        } else if (status_byte & 0x01) != 0 || pressure > 0 {
            "Contact".to_string()
        } else {
            "Hover".to_string()
        };

        // Treat (0,0) as invalid/out of range, even if status says otherwise
        let is_connected = status != "Out of Range" && (x != 0 || y != 0);

        Some(TabletData {
            status,
            x,
            y,
            pressure,
            tilt_x: 0,
            tilt_y: 0,
            buttons: 0,
            eraser: false,
            hover_distance: 0,
            raw_data: raw,
            is_connected,
            ..Default::default()
        })
    }
}
