use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct TenMoonParser;

impl ReportParser for TenMoonParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 13 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        if data[11] != 0xFF {
            // Aux Report
            let mut buttons: u8 = 0;
            // Pack the first 8 aux buttons into the u8 `buttons` field
            if data[12] == 0x31 {
                buttons |= 1 << 0;
            }
            if data[12] == 0x33 && (data[11] & 0x80) == 0 {
                buttons |= 1 << 1;
            }
            if data[12] == 0x33 && (data[11] & 0x40) == 0 {
                buttons |= 1 << 2;
            }
            if data[12] == 0x33 && (data[11] & 0x20) == 0 {
                buttons |= 1 << 3;
            }
            if data[12] == 0x33 && (data[11] & 0x10) == 0 {
                buttons |= 1 << 4;
            }
            if data[12] == 0x33 && (data[11] & 0x08) == 0 {
                buttons |= 1 << 5;
            }
            if data[12] == 0x23 {
                buttons |= 1 << 6;
            }
            if data[12] == 0x32 {
                buttons |= 1 << 7;
            }
            // Some buttons will be dropped because `TabletData.buttons` only holds 8 bits:
            // 9th: 0x13, 10th: 0x33 && !Bit 0, 11th: 0x33 && !Bit 1, 12th: 0x33 && !Bit 2

            Some(TabletData {
                status: "Aux".to_string(),
                buttons,
                raw_data: raw,
                is_connected: true,
                ..Default::default()
            })
        } else {
            // Tablet Report
            let x = ((data[1] as u16) << 8) | (data[2] as u16);
            let raw_y = ((data[3] as u16) << 8) | (data[4] as u16);
            let y = raw_y; // since it's u16, it can't be negative, so Max(..., 0) is trivial

            let btn_pressed = (data[9] & 6) != 0;
            let pre_pressure = ((data[5] as u16) << 8) | (data[6] as u16);
            let pressure_offset = if btn_pressed { 50 } else { 0 };

            let pressure = if pre_pressure >= pressure_offset {
                let adjusted = pre_pressure - pressure_offset;
                if 0x0672 > adjusted {
                    0x0672 - adjusted
                } else {
                    0
                }
            } else {
                0x0672
            };

            let mut buttons: u8 = 0;
            if (data[9] & 0x04) != 0 {
                buttons |= 1 << 0;
            }
            if (data[9] & 6) == 6 {
                buttons |= 1 << 1;
            }

            let status = if pressure > 0 { "Contact" } else { "Hover" };

            Some(TabletData {
                status: status.to_string(),
                x,
                y,
                pressure,
                buttons,
                raw_data: raw,
                is_connected: true,
                ..Default::default()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenmoon_tablet() {
        let parser = TenMoonParser;
        let data: [u8; 13] = [0, 1, 2, 3, 4, 0x05, 0x06, 0, 0, 4, 0, 0xFF, 0];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.y, 772);
        assert_eq!(report.pressure, 414); // 1650 - (1286 - 50)
        assert_eq!(report.buttons, 1);
    }
}
