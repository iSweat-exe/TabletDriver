use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct BambooPadParser;

impl ReportParser for BambooPadParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 24 {
            return None;
        }

        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");

        if data[0] == 0x10 {
            if data[1] == 0x01 {
                // Tablet Report
                let x = u16::from_le_bytes([data[3], data[4]]);
                let y = u16::from_le_bytes([data[5], data[6]]);
                let pressure = u16::from_le_bytes([data[7], data[8]]);

                let mut buttons: u8 = 0;
                if (data[2] & 0x02) != 0 { buttons |= 1 << 0; }
                let eraser = (data[2] & 0x08) != 0;

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
            } else if data[1] == 0x06 {
                // Aux Report
                let mut buttons: u8 = 0;
                if data[23] == 1 { buttons |= 1 << 0; }
                if data[23] == 2 { buttons |= 1 << 1; }

                Some(TabletData {
                    status: "Aux".to_string(),
                    buttons,
                    raw_data: raw,
                    is_connected: true,
                    ..Default::default()
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
