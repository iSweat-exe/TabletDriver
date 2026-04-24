use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

// Intuos V2

pub struct IntuosV2Parser;

impl IntuosV2Parser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IntuosV2Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl IntuosV2Parser {
    fn parse_internal(&self, data: &[u8], raw: String) -> Option<TabletData> {
        match data[0] {
            0x10 => self.parse_tablet(data, raw, false),
            0x1E => self.parse_tablet(data, raw, true),
            0x11 => self.parse_aux(data, raw),
            _ => None,
        }
    }

    fn parse_tablet(&self, data: &[u8], raw: String, offset: bool) -> Option<TabletData> {
        let min_len = if offset { 13 } else { 17 };
        if data.len() < min_len {
            return None;
        }

        let (x_low, x_high, y_low, y_high, p_low, p_high, tx, ty, btn_byte, eraser_bit) = if offset
        {
            (
                data[3], data[5], data[6], data[8], data[9], data[10], data[11], data[12], data[2],
                4,
            )
        } else {
            (
                data[2], data[4], data[5], data[7], data[8], data[9], data[10], data[11], data[1],
                4,
            )
        };

        let x = (x_low as u32) | ((x_high as u32) << 16);
        let y = (y_low as u32) | ((y_high as u32) << 16);
        let pressure = (p_low as u16) | ((p_high as u16) << 8);

        let tilt_x = tx as i8;
        let tilt_y = ty as i8;

        let mut buttons: u8 = 0;
        if (btn_byte & 0x02) != 0 {
            buttons |= 1 << 0;
        }
        if (btn_byte & 0x04) != 0 {
            buttons |= 1 << 1;
        }
        if offset && (btn_byte & 0x08) != 0 {
            buttons |= 1 << 2;
        }

        let eraser = (btn_byte & (1 << eraser_bit)) != 0;
        let status = if pressure > 0 { "Contact" } else { "Hover" };
        let hover_distance = if offset { data[11] } else { data[16] };

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
        if data.len() < 2 {
            return None;
        }
        let buttons = data[1];
        Some(TabletData {
            status: "Aux".to_string(),
            buttons,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }
}

impl ReportParser for IntuosV2Parser {
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

pub struct WacomDriverIntuosV2Parser {
    inner: IntuosV2Parser,
}

impl WacomDriverIntuosV2Parser {
    pub fn new() -> Self {
        Self {
            inner: IntuosV2Parser::new(),
        }
    }
}

impl Default for WacomDriverIntuosV2Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportParser for WacomDriverIntuosV2Parser {
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
    fn test_intuos_v2_tablet_report() {
        let parser = IntuosV2Parser::new();
        // Report ID 0x10, X/Y/Pressure/Tilt/Buttons
        let mut data = [0u8; 17];
        data[0] = 0x10;
        data[1] = 0x02; // Pen Button 1
        data[2] = 0x34; // X low
        data[4] = 0x12; // X high
        data[8] = 0xFF; // Pressure low
        data[9] = 0x03; // Pressure high

        let result = parser.parse(&data).expect("Should parse");
        assert_eq!(result.buttons, 1 << 0);
        assert_eq!(result.pressure, 1023);
    }
}
