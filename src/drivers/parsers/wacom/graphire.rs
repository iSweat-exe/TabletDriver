use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct GraphireParser;

impl ReportParser for GraphireParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 8 {
            return None;
        }

        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");

        if data[0] == 0x02 {
            let pos_available = (data[1] & 0x80) != 0
                || data[2] != 0 || data[3] != 0 || data[4] != 0 || data[5] != 0
                || (data[6] as u16 | (((data[7] & 0x03) as u16) << 8)) != 0;

            if pos_available {
                if (data[1] & 0x40) != 0 {
                    // Mouse Report
                    let x = u16::from_le_bytes([data[2], data[3]]);
                    let y = u16::from_le_bytes([data[4], data[5]]);

                    // Aux Buttons inline mouse mapping
                    let mut buttons: u8 = 0;
                    if (data[7] & 0x40) != 0 { buttons |= 1 << 0; }
                    if (data[7] & 0x80) != 0 { buttons |= 1 << 1; }

                    return Some(TabletData {
                        status: "Mouse".to_string(),
                        x,
                        y,
                        buttons,
                        raw_data: raw,
                        is_connected: true,
                        ..Default::default()
                    });
                }

                // Tablet Report
                let x = u16::from_le_bytes([data[2], data[3]]);
                let y = u16::from_le_bytes([data[4], data[5]]);

                let pressure = if (data[1] & 0x01) != 0 {
                    (data[6] as u16) | (((data[7] & 0x03) as u16) << 8)
                } else {
                    0
                };

                let eraser = (data[1] & 0x20) != 0;

                let mut buttons: u8 = 0;
                // Pen Buttons (0, 1) mapped from bit 1 & 2
                if (data[1] & 0x02) != 0 { buttons |= 1 << 0; }
                if (data[1] & 0x04) != 0 { buttons |= 1 << 1; }
                // Aux Buttons appended to standard buttons variable
                if (data[7] & 0x40) != 0 { buttons |= 1 << 2; }
                if (data[7] & 0x80) != 0 { buttons |= 1 << 3; }

                let status = if pressure > 0 { "Contact" } else { "Hover" };

                return Some(TabletData {
                    status: status.to_string(),
                    x,
                    y,
                    pressure,
                    buttons,
                    eraser,
                    raw_data: raw,
                    is_connected: true,
                    ..Default::default()
                });
            }

            // Aux Report
            let mut buttons: u8 = 0;
            if (data[7] & 0x40) != 0 { buttons |= 1 << 0; }
            if (data[7] & 0x80) != 0 { buttons |= 1 << 1; }

            Some(TabletData {
                status: "Aux".to_string(),
                buttons,
                raw_data: raw,
                is_connected: true,
                ..Default::default()
            })
        } else {
            None
        }
    }
}
