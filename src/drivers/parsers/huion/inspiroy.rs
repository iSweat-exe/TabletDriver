use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct InspiroyParser;

impl ReportParser for InspiroyParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 8 {
            return None;
        }

        let raw = data
            .iter()
            .take(14)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        match data[1] {
            0x00 => return None, // OutOfRange
            0xE0 | 0xE3 => {
                // Aux Report
                if data.len() < 7 {
                    return None;
                }
                let buttons = data[4]; // first 8 buttons
                return Some(TabletData {
                    status: "Aux".to_string(),
                    buttons,
                    raw_data: raw,
                    is_connected: true,
                    ..Default::default()
                });
            }
            0xF1 | 0xF0 => {
                // Wheel Report
                return Some(TabletData {
                    status: "Aux".to_string(),
                    buttons: 0,
                    raw_data: raw,
                    is_connected: true,
                    ..Default::default()
                });
            }
            _ => {}
        }

        // Huion/Inspiroy standard report style (Giano)
        // Position X: [2] | [3] << 8 | ([8] & 1) << 16
        // Position Y: [4] | [5] << 8 | ([9] & 1) << 16
        // Pressure: [6] | [7] << 8

        let x = (data[2] as u32)
            | ((data[3] as u32) << 8)
            | ((data.get(8).unwrap_or(&0) & 1) as u32) << 16;
        let y = (data[4] as u32)
            | ((data[5] as u32) << 8)
            | ((data.get(9).unwrap_or(&0) & 1) as u32) << 16;
        let pressure = (data[6] as u16) | ((data[7] as u16) << 8);

        // Tilt (X at 10, Y at 11) - OTD uses * -1 for Giano
        let tilt_x = if data.len() >= 11 {
            (data[10] as i8).wrapping_mul(-1)
        } else {
            0
        };
        let tilt_y = if data.len() >= 12 {
            (data[11] as i8).wrapping_mul(-1)
        } else {
            0
        };

        // Buttons (Byte 1)
        let buttons = (data[1] >> 1) & 0x07; // Bits 1, 2, 3
        let eraser = (data[1] & 0x10) != 0; // OTD sometimes uses bit 4 for eraser or similar

        let status = if pressure > 0 {
            "Contact".to_string()
        } else if (data[1] & 0x01) != 0 {
            "Hover".to_string()
        } else {
            "Out of Range".to_string()
        };

        Some(TabletData {
            status,
            x: x as u16, // Assuming we keep u16 for now, but 3 bytes might exceed.
            // OTD uses uint for position. Let's check if we should upgrade TabletData
            y: y as u16,
            pressure,
            tilt_x,
            tilt_y,
            buttons,
            eraser,
            hover_distance: 0,
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
    fn test_inspiroy_tablet() -> Result<(), Box<dyn std::error::Error>> {
        let parser = InspiroyParser;
        let data: [u8; 12] = [0x08, 0x81, 0x02, 0x01, 0, 0x04, 0x03, 0, 0x01, 0x00, 0, 0];
        let report = parser
            .parse(&data)
            .ok_or("Inspiroy parser failed to parse tablet packet")?;
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.pressure, 3);
        Ok(())
    }
}
