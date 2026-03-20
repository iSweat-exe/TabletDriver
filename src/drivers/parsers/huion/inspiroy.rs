use crate::drivers::parsers::ReportParser;
use crate::drivers::TabletData;

pub struct InspiroyParser;

impl ReportParser for InspiroyParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 8 {
            return None;
        }

        // Huion/Inspiroy standard report style (Giano)
        // Position X: [2] | [3] << 8 | ([8] & 1) << 16
        // Position Y: [4] | [5] << 8 | ([9] & 1) << 16
        // Pressure: [6] | [7] << 8

        let x = (data[2] as u32) | ((data[3] as u32) << 8) | ((data[8] as u32 & 1) << 16);
        let y = (data[4] as u32) | ((data[5] as u32) << 8) | ((data[9] as u32 & 1) << 16);
        let pressure = (data[6] as u16) | ((data[7] as u16) << 8);

        // Tilt (X at 10, Y at 11) - OTD uses * -1 for Giano
        let tilt_x = if data.len() >= 11 {
            (data[10] as i8).wrapping_mul(-1)
        } else {
            0
        };
        let tilt_y = if data.len() >= 12 {
            (data[11] as i8).wrapping_mul(-1)
        } else {
            0
        };

        // Buttons (Byte 1)
        let buttons = (data[1] >> 1) & 0x07; // Bits 1, 2, 3
        let eraser = (data[1] & 0x10) != 0; // OTD sometimes uses bit 4 for eraser or similar

        let raw = data
            .iter()
            .take(14)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let status = if pressure > 0 {
            "Contact".to_string()
        } else if (data[1] & 0x01) != 0 {
            "Hover".to_string()
        } else {
            "Out of Range".to_string()
        };

        Some(TabletData {
            status,
            x: x as u16, // Assuming we keep u16 for now, but 3 bytes might exceed.
            // OTD uses uint for position. Let's check if we should upgrade TabletData
            y: y as u16,
            pressure,
            tilt_x,
            tilt_y,
            buttons,
            eraser,
            hover_distance: 0,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }
}
