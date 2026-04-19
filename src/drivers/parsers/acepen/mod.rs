use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct AcepenParser {
    aux_state: AtomicU8,
}

impl AcepenParser {
    pub fn new() -> Self {
        Self {
            aux_state: AtomicU8::new(0),
        }
    }
}

impl ReportParser for AcepenParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 11 {
            return None;
        }

        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        match data[1] {
            0x41 => {
                // PEN_MODE
                if (data[2] & 0xF0) == 0xA0 {
                    let x = u16::from_le_bytes([data[3], data[4]]);
                    let y = u16::from_le_bytes([data[5], data[6]]);
                    let pressure = u16::from_le_bytes([data[7], data[8]]);

                    let mut buttons: u8 = 0;
                    if (data[2] & 0x02) != 0 {
                        buttons |= 1 << 0;
                    }
                    if (data[2] & 0x04) != 0 {
                        buttons |= 1 << 1;
                    }

                    let tilt_x = data[9] as i8;
                    let tilt_y = data[10] as i8;

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
                } else {
                    None
                }
            }
            0x42 => {
                // AUX_MODE
                let bit_index = if data[4] > 0 {
                    data[4].trailing_zeros()
                } else {
                    0
                };
                let is_set = (data[3] & 0x01) != 0;

                let mut current_state = self.aux_state.load(Ordering::Relaxed);
                if is_set {
                    current_state |= 1 << bit_index;
                } else {
                    current_state &= !(1 << bit_index);
                }
                self.aux_state.store(current_state, Ordering::Relaxed);

                Some(TabletData {
                    status: "Aux".to_string(),
                    buttons: current_state,
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
    fn test_acepen_pen_contact() {
        let parser = AcepenParser::new();
        let data: [u8; 11] = [0, 0x41, 0xA2, 0x02, 0x01, 0x04, 0x03, 0x01, 0x00, 10, 20];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.y, 772);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 1);
        assert_eq!(report.tilt_x, 10);
        assert_eq!(report.tilt_y, 20);
    }

    #[test]
    fn test_acepen_aux() {
        let parser = AcepenParser::new();
        let data: [u8; 11] = [0, 0x42, 0, 1, 4, 0, 0, 0, 0, 0, 0];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Aux");
        assert_eq!(report.buttons, 4);
    }
}
