use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

// --- Intuos V3 ---

pub struct IntuosV3Parser;

impl IntuosV3Parser {
    pub fn new() -> Self {
        Self
    }

    fn parse_internal(&self, data: &[u8], raw: String) -> Option<TabletData> {
        match data[0] {
            0x11 => self.parse_aux(data, raw),
            0x1E => self.parse_extended(data, raw),
            0x1F if data.len() > 1 && data[1] == 0x01 => self.parse_tablet(data, raw),
            _ => None,
        }
    }

    fn parse_tablet(&self, data: &[u8], raw: String) -> Option<TabletData> {
        if data.len() < 14 {
            return None;
        }
        let x = u16::from_le_bytes([data[3], data[4]]);
        let y = u16::from_le_bytes([data[5], data[6]]);
        let pressure = u16::from_le_bytes([data[7], data[8]]);

        let tilt_x = if (data[9] & 0x80) != 0 {
            (data[9] as i16 - 0xFF) as i8
        } else {
            data[9] as i8
        };
        let tilt_y = if (data[11] & 0x80) != 0 {
            (data[11] as i16 - 0xFF) as i8
        } else {
            data[11] as i8
        };

        let mut buttons: u8 = 0;
        if (data[2] & 0x02) != 0 {
            buttons |= 1 << 0;
        }
        if (data[2] & 0x04) != 0 {
            buttons |= 1 << 1;
        }
        let eraser = (data[2] & 0x20) != 0;

        let status = if pressure > 0 { "Contact" } else { "Hover" };
        let hover_distance = data[13];

        Some(TabletData {
            status: status.to_string(),
            x,
            y,
            pressure,
            tilt_x,
            tilt_y,
            buttons,
            eraser,
            hover_distance,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }

    fn parse_extended(&self, data: &[u8], raw: String) -> Option<TabletData> {
        if data.len() < 20 {
            return None;
        }
        let x = (u16::from_le_bytes([data[3], data[4]]) as u32) | ((data[5] as u32) << 16);
        let y = (u16::from_le_bytes([data[6], data[7]]) as u32) | ((data[8] as u32) << 16);
        let pressure = u16::from_le_bytes([data[9], data[10]]);
        let tilt_x = (i16::from_le_bytes([data[11], data[12]])) as i8;
        let tilt_y = (i16::from_le_bytes([data[13], data[14]])) as i8;

        let mut buttons: u8 = 0;
        if (data[2] & 0x02) != 0 {
            buttons |= 1 << 0;
        }
        if (data[2] & 0x04) != 0 {
            buttons |= 1 << 1;
        }
        if (data[2] & 0x08) != 0 {
            buttons |= 1 << 2;
        }
        let eraser = (data[2] & 0x20) != 0;

        let status = if pressure > 0 { "Contact" } else { "Hover" };
        let hover_distance = data[19];

        Some(TabletData {
            status: status.to_string(),
            x: x.min(0xFFFF) as u16,
            y: y.min(0xFFFF) as u16,
            pressure,
            tilt_x,
            tilt_y,
            buttons,
            eraser,
            hover_distance,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }

    fn parse_aux(&self, data: &[u8], raw: String) -> Option<TabletData> {
        if data.len() < 4 {
            return None;
        }
        let mut buttons: u16 = 0;
        let b1 = data[1];
        let b2 = data[3];
        if (b1 & 1) != 0 {
            buttons |= 1 << 0;
        }
        if (b1 & 2) != 0 {
            buttons |= 1 << 1;
        }
        if (b1 & 4) != 0 {
            buttons |= 1 << 2;
        }
        if (b1 & 8) != 0 {
            buttons |= 1 << 3;
        }
        if (b2 & 1) != 0 {
            buttons |= 1 << 4;
        }
        if (b1 & 16) != 0 {
            buttons |= 1 << 5;
        }
        if (b1 & 32) != 0 {
            buttons |= 1 << 6;
        }
        if (b1 & 64) != 0 {
            buttons |= 1 << 7;
        }
        // higher bits truncated to u8 buttons in TabletData

        Some(TabletData {
            status: "Aux".to_string(),
            buttons: buttons as u8,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }
}

impl ReportParser for IntuosV3Parser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.is_empty() {
            return None;
        }
        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        self.parse_internal(data, raw)
    }
}

pub struct WacomDriverIntuosV3Parser {
    inner: IntuosV3Parser,
}

impl WacomDriverIntuosV3Parser {
    pub fn new() -> Self {
        Self {
            inner: IntuosV3Parser::new(),
        }
    }
}

impl ReportParser for WacomDriverIntuosV3Parser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }
        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        self.inner.parse_internal(&data[1..], raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intuos_v3_tablet_report() {
        let parser = IntuosV3Parser::new();
        // Report ID 1F, status 01, Pen buttons, X, Y, Pressure, Tilt
        let mut data = [0u8; 15];
        data[0] = 0x1F;
        data[1] = 0x01;
        data[2] = 0x02; // Pen button 1
        data[3] = 0x01;
        data[4] = 0x00; // X = 1
        data[7] = 0xAA;
        data[8] = 0x00; // Pressure = 170

        let result = parser.parse(&data).expect("Should parse");
        assert_eq!(result.x, 1);
        assert_eq!(result.pressure, 170);
        assert_eq!(result.buttons, 1 << 0);
    }
}
