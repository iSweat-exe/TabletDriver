use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct ViewSonicParser;

impl ReportParser for ViewSonicParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 14 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        if (data[9] & 0b11) != 0b11 {
            return None; // Ignored report
        }

        let x = u16::from_le_bytes([data[1], data[2]]);
        let y = u16::from_le_bytes([data[5], data[6]]);

        let pressure = if (data[9] & 0x04) != 0 {
            u16::from_le_bytes([data[10], data[11]])
        } else {
            0
        };

        let mut buttons: u8 = 0;
        if (data[9] & 0x08) != 0 {
            buttons |= 1 << 0;
        }
        if (data[9] & 0x10) != 0 {
            buttons |= 1 << 1;
        }

        let tilt_x = data[12] as i8;
        let tilt_y = data[13] as i8;

        let status = if pressure > 0 { "Contact" } else { "Hover" };

        Some(TabletData {
            status: status.to_string(),
            x,
            y,
            pressure,
            tilt_x,
            tilt_y,
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
    fn test_viewsonic_tablet() {
        let parser = ViewSonicParser;
        let data: [u8; 14] = [
            0, 0x02, 0x01, 0, 0, 0x04, 0x03, 0, 0, 0x1F, 0x01, 0x00, 10, 20,
        ];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 3);
        assert_eq!(report.tilt_x, 10);
    }
}
