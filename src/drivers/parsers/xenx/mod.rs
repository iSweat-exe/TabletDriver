use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct XenxParser;

impl ReportParser for XenxParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        match data[0] {
            0x01 => {
                // Tablet Report
                if data[1] == 0 {
                    return None; // Out of range
                }

                if data.len() < 8 {
                    return None;
                }
                let x = u16::from_le_bytes([data[2], data[3]]);
                let y = u16::from_le_bytes([data[4], data[5]]);
                let pressure = u16::from_le_bytes([data[6], data[7]]);

                let mut buttons: u8 = 0;
                if (data[1] & 0x02) != 0 {
                    buttons |= 1 << 0;
                }
                if (data[1] & 0x04) != 0 {
                    buttons |= 1 << 1;
                }
                let eraser = (data[1] & 0x40) != 0;

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
            }
            0x02 => {
                // Aux Report
                if data.len() < 12 {
                    return None;
                }
                let mut buttons: u8 = 0;

                // data[2..11] are booleans for aux buttons
                for i in 0..8 {
                    if data[2 + i] != 0 {
                        buttons |= 1 << i;
                    }
                }

                Some(TabletData {
                    status: "Aux".to_string(),
                    buttons,
                    raw_data: raw,
                    is_connected: true,
                    ..Default::default()
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xenx_tablet() {
        let parser = XenxParser;
        let data: [u8; 8] = [0x01, 0x06, 0x02, 0x01, 0x04, 0x03, 0x01, 0x00];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 3);
    }
}
