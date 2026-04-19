use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct RobotPenParser;

impl ReportParser for RobotPenParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 12 || data[1] != 0x42 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let x = u16::from_le_bytes([data[6], data[7]]);
        let y = u16::from_le_bytes([data[8], data[9]]);
        let pressure = u16::from_le_bytes([data[10], data[11]]);

        let mut buttons: u8 = 0;
        if (data[11] & 0x02) != 0 {
            buttons |= 1 << 0;
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
    fn test_robotpen_tablet() {
        let parser = RobotPenParser;
        let data: [u8; 12] = [
            0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x04, 0x03, 0x01, 0x02,
        ];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.y, 772);
        assert_eq!(report.pressure, 513); // 2 | 1<<8
        assert_eq!(report.buttons, 1); // data[11] & 2 = 2 => btn0 set
    }
}
