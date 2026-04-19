use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct FallbackParser;

impl ReportParser for FallbackParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        // Generic HID layout: [ReportID, Status, X_lo, X_hi, Y_lo, Y_hi, P_lo, P_hi, ...]

        if data.len() < 8 {
            return None;
        }

        let x = ((data[3] as u16) << 8) | (data[2] as u16);
        let y = ((data[5] as u16) << 8) | (data[4] as u16);
        let pressure = ((data[7] as u16) << 8) | (data[6] as u16);

        let raw = data
            .iter()
            .take(10)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        // Status byte conventions: 0xC0/0x00 = out of range, bit 0 = tip contact
        let status_byte = data[1];

        let status = if status_byte == 0xC0 || status_byte == 0x00 {
            "Out of Range".to_string()
        } else if (status_byte & 0x01) != 0 || pressure > 0 {
            "Contact".to_string()
        } else {
            "Hover".to_string()
        };

        let is_connected = status != "Out of Range";

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
