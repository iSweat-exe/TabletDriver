use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct PTUParser;

impl ReportParser for PTUParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 8 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        if data[0] == 0x02 {
            // Tablet Report
            let x = u16::from_le_bytes([data[2], data[3]]);
            let y = u16::from_le_bytes([data[4], data[5]]);
            let pressure = u16::from_le_bytes([data[6], data[7]]);

            let mut buttons: u8 = 0;
            if (data[1] & 0x02) != 0 {
                buttons |= 1 << 0;
            }
            if (data[1] & 0x10) != 0 {
                buttons |= 1 << 1;
            }

            let eraser = (data[1] & 0x04) != 0;

            let status = if pressure > 0 { "Contact" } else { "Hover" };

            Some(TabletData {
                status: status.to_string(),
                x,
                y,
                pressure,
                buttons,
                eraser,
                raw_data: raw,
                is_connected: true,
                ..Default::default()
            })
        } else {
            None
        }
    }
}
