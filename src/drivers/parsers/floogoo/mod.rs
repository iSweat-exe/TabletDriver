use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct FlooGooParser;

impl ReportParser for FlooGooParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 12 || data[0] != 0x01 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        // Bit 5 of data[1] is set when pen is in range
        if (data[1] & 0x20) == 0 {
            return None; // Out of range
        }

        let x = u16::from_le_bytes([data[2], data[3]]);
        let y = u16::from_le_bytes([data[4], data[5]]);
        let pressure = u16::from_le_bytes([data[6], data[7]]);

        // Unit is [-9000..9000]x10^-3 degrees = [-90..=90] degrees. Cast direct to i8
        let raw_tilt_x = i16::from_le_bytes([data[8], data[9]]);
        let raw_tilt_y = i16::from_le_bytes([data[10], data[11]]);
        let tilt_x = (raw_tilt_x as f32 * 0.01).round() as i8;
        let tilt_y = (raw_tilt_y as f32 * 0.01).round() as i8;

        let mut buttons: u8 = 0;
        if (data[1] & 0x02) != 0 {
            buttons |= 1 << 0;
        }
        if (data[1] & 0x04) != 0 {
            buttons |= 1 << 1;
        }

        let eraser = (data[1] & 0x08) != 0;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floogoo_pen_contact() -> Result<(), Box<dyn std::error::Error>> {
        let parser = FlooGooParser;
        let data: [u8; 12] = [
            0x01, 0x2A, 0x02, 0x01, 0x04, 0x03, 0x05, 0x00, 0xE8, 0x03, 0x18, 0xFC,
        ];
        let report = parser
            .parse(&data)
            .ok_or("FlooGoo parser failed to parse tablet packet")?;
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.y, 772);
        assert_eq!(report.pressure, 5);
        assert_eq!(report.buttons, 1); // 0x02 is bit 0
        assert!(report.eraser); // 0x08 is eraser
        assert_eq!(report.tilt_x, 10);
        assert_eq!(report.tilt_y, -10);
        Ok(())
    }

    #[test]
    fn test_floogoo_out_of_range() {
        let parser = FlooGooParser;
        let data: [u8; 12] = [0x01, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(parser.parse(&data).is_none());
    }
}
