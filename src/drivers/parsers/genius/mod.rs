use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct GeniusParserV1;

impl ReportParser for GeniusParserV1 {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.is_empty() {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        match data[0] {
            0x10 => {
                // Tablet Report
                if data.len() < 8 {
                    return None;
                }
                let x = u16::from_le_bytes([data[1], data[2]]);
                let y = u16::from_le_bytes([data[3], data[4]]);
                let pressure = if (data[5] & 0x04) != 0 {
                    u16::from_le_bytes([data[6], data[7]])
                } else {
                    0
                };

                let mut buttons: u8 = 0;
                if (data[5] & 0x08) != 0 {
                    buttons |= 1 << 0;
                }
                if (data[5] & 0x10) != 0 {
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
            0x11 => {
                // Mouse Report
                if data.len() < 7 {
                    return None;
                }
                let x = u16::from_le_bytes([data[2], data[3]]);
                let y = u16::from_le_bytes([data[4], data[5]]);

                let mut buttons: u8 = 0;
                if (data[1] & 0x01) != 0 {
                    buttons |= 1 << 0;
                }
                if (data[1] & 0x02) != 0 {
                    buttons |= 1 << 1;
                }
                if (data[1] & 0x04) != 0 {
                    buttons |= 1 << 2;
                }

                // data[6] is Y scroll
                Some(TabletData {
                    status: "Mouse".to_string(),
                    x,
                    y,
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

pub struct GeniusParserV2;

impl ReportParser for GeniusParserV2 {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.is_empty() {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        match data[0] {
            0x02 => {
                // Tablet Report
                if data.len() < 8 {
                    return None;
                }
                let x = u16::from_le_bytes([data[1], data[2]]);
                let y = u16::from_le_bytes([data[3], data[4]]);
                let pressure = if (data[5] & 0x04) != 0 {
                    u16::from_le_bytes([data[6], data[7]])
                } else {
                    0
                };

                let mut buttons: u8 = 0;
                if (data[5] & 0x08) != 0 {
                    buttons |= 1 << 0;
                }
                if (data[5] & 0x10) != 0 {
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
            0x05 => {
                // Aux Report
                if data.len() < 4 {
                    return None;
                }
                let aux_byte = data[3];
                let mut buttons: u8 = 0;

                if aux_byte > 0 {
                    let active_index = (aux_byte - 1) / 2;
                    if active_index < 8 {
                        buttons |= 1 << active_index;
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
    fn test_genius_v1_tablet() {
        let parser = GeniusParserV1;
        // data[0]=0x10. x=258, y=772, pressure=1
        // data[5] = 0x0C (0x04 pressure valid | 0x08 btn0)
        let data: [u8; 8] = [0x10, 0x02, 0x01, 0x04, 0x03, 0x0C, 0x01, 0x00];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.y, 772);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 1);
    }

    #[test]
    fn test_genius_v2_tablet() {
        let parser = GeniusParserV2;
        let data: [u8; 8] = [0x02, 0x02, 0x01, 0x04, 0x03, 0x0C, 0x01, 0x00];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
    }
}
