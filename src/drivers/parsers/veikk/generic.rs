use crate::drivers::parsers::ReportParser;
use crate::drivers::TabletData;

pub struct VeikkParser;

impl ReportParser for VeikkParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 11 {
            return None;
        }

        // Veikk standard report style
        // Position X: [3] | [4] << 8 | [5] << 16
        // Position Y: [6] | [7] << 8 | [8] << 16
        // Pressure: [9] | [10] << 8

        let x = (data[3] as u32) | ((data[4] as u32) << 8) | ((data[5] as u32) << 16);
        let y = (data[6] as u32) | ((data[7] as u32) << 8) | ((data[8] as u32) << 16);
        let pressure = (data[9] as u16) | ((data[10] as u16) << 8);

        // Buttons (Byte 2)
        // Bits 1 and 2
        let buttons = (data[2] >> 1) & 0x03;

        let raw = data
            .iter()
            .take(14)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let status = if (data[2] & 0x20) != 0 {
            if pressure > 0 {
                "Contact"
            } else {
                "Hover"
            }
        } else {
            "Out of Range"
        };

        Some(TabletData {
            status: status.to_string(),
            x: x as u16,
            y: y as u16,
            pressure,
            tilt_x: 0,
            tilt_y: 0,
            buttons,
            eraser: false,
            hover_distance: 0,
            raw_data: raw,
            is_connected: true,
        })
    }
}
