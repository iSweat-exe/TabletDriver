use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct XenceLabsParser;

impl ReportParser for XenceLabsParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let report_byte = data[1];

        if (report_byte & 0xF0) == 0xF0 {
            // XP_PenAuxReport style
            if data.len() < 3 {
                return None;
            }
            Some(TabletData {
                status: "Aux".to_string(),
                buttons: data[2], // Default XP_PenAuxReport maps data[2] to first 8 buttons
                raw_data: raw,
                is_connected: true,
                ..Default::default()
            })
        } else if (report_byte & 0x20) != 0 {
            // XenceLabsTabletReport
            if data.len() < 10 {
                return None;
            }
            let x = u16::from_le_bytes([data[2], data[3]]);
            let y = u16::from_le_bytes([data[4], data[5]]);
            let pressure = u16::from_le_bytes([data[6], data[7]]);

            let mut buttons: u8 = 0;
            if (report_byte & 0x02) != 0 {
                buttons |= 1 << 0;
            }
            if (report_byte & 0x04) != 0 {
                buttons |= 1 << 1;
            }
            if (report_byte & 0x08) != 0 {
                buttons |= 1 << 2;
            }

            let eraser = (report_byte & 0x40) != 0;

            let tilt_x = data[8] as i8;
            let tilt_y = data[9] as i8;

            let status = if pressure > 0 { "Contact" } else { "Hover" };

            Some(TabletData {
                status: status.to_string(),
                x,
                y,
                pressure,
                tilt_x,
                tilt_y,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xencelabs_tablet() {
        let parser = XenceLabsParser;
        let data: [u8; 10] = [0, 0x2E, 0x02, 0x01, 0x04, 0x03, 0x01, 0x00, 10, 20];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 7);
        assert_eq!(report.tilt_x, 10);
    }
}
