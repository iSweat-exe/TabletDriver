use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct BostoParser;

impl ReportParser for BostoParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 8 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        if data[1] == 0x00 {
            return None; // Out of range report
        }

        let x = u16::from_le_bytes([data[2], data[3]]);
        let y = u16::from_le_bytes([data[4], data[5]]);
        let pressure = u16::from_le_bytes([data[6], data[7]]);

        let mut buttons: u8 = 0;
        if (data[1] & 0x20) != 0 {
            buttons |= 1 << 0;
        }
        if (data[1] & 0x02) != 0 {
            buttons |= 1 << 1;
        }

        let status = if pressure > 0 { "Contact" } else { "Hover" };

        Some(TabletData {
            status: status.to_string(),
            x,
            y,
            pressure,
            tilt_x: 0,
            tilt_y: 0,
            buttons,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bosto_pen_contact() {
        let parser = BostoParser;
        let data: [u8; 8] = [0, 0x22, 0x02, 0x01, 0x04, 0x03, 0x01, 0x00];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.y, 772);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 3); // 0x20 and 0x02 map to bits 0 and 1
    }

    #[test]
    fn test_bosto_out_of_range() {
        let parser = BostoParser;
        let data: [u8; 8] = [0, 0x00, 0x02, 0x01, 0x04, 0x03, 0x01, 0x00];
        assert!(parser.parse(&data).is_none());
    }
}
