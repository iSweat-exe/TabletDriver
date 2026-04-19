use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct LifetecParser;

impl ReportParser for LifetecParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 8 || data[0] != 0x02 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let x = u16::from_le_bytes([data[1], data[2]]);
        let y = u16::from_le_bytes([data[3], data[4]]);
        let pressure = u16::from_le_bytes([data[6], data[7]]);

        let mut buttons: u8 = 0;
        if (data[5] & 0x08) != 0 { buttons |= 1 << 0; }
        if (data[5] & 0x10) != 0 { buttons |= 1 << 1; }

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
    fn test_lifetec_tablet() {
        let parser = LifetecParser;
        let data: [u8; 8] = [0x02, 0x02, 0x01, 0x04, 0x03, 0x08, 0x01, 0x00];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.y, 772);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 1);
    }

    #[test]
    fn test_lifetec_invalid() {
        let parser = LifetecParser;
        let data: [u8; 8] = [0x03, 0x02, 0x01, 0x04, 0x03, 0x08, 0x01, 0x00];
        assert!(parser.parse(&data).is_none());
    }
}
