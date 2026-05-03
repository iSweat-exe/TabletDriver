use crate::drivers::TabletData;
use crate::drivers::parsers::{ReportParser, xp_pen::standard::parse as standard_parse};

fn parse_aux(data: &[u8], raw: String, offset: usize) -> Option<TabletData> {
    if data.len() < offset + 1 {
        return None;
    }
    let buttons = data[offset]; // grab first byte of aux buttons
    Some(TabletData {
        status: "Aux".to_string(),
        buttons,
        raw_data: raw,
        is_connected: true,
        ..Default::default()
    })
}

fn parse_gen2(data: &[u8], raw: String) -> Option<TabletData> {
    if data.len() < 14 {
        return None;
    }
    let x = (data[2] as u32) | ((data[10] as u32) << 16);
    let y = (data[4] as u32) | ((data[11] as u32) << 16);
    let pressure =
        (u16::from_le_bytes([data[6], data[7]]) & 0xBFFF) | (((data[13] & 0x01) as u16) << 13);
    let tilt_x = data[8] as i8;
    let tilt_y = data[9] as i8;

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
        x: x.min(0xFFFF) as u16,
        y: y.min(0xFFFF) as u16,
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

fn parse_offset_pressure(data: &[u8], raw: String, has_tilt: bool) -> Option<TabletData> {
    if data.len() < 8 {
        return None;
    }
    let x = u16::from_le_bytes([data[2], data[3]]);
    let y = u16::from_le_bytes([data[4], data[5]]);
    let pressure = u16::from_le_bytes([data[6], data[7]]); // Pressure offset uses same index as std

    let mut buttons: u8 = 0;
    if (data[1] & 0x02) != 0 {
        buttons |= 1 << 0;
    }
    if (data[1] & 0x04) != 0 {
        buttons |= 1 << 1;
    }
    let eraser = (data[1] & 0x08) != 0;

    let mut tilt_x = 0;
    let mut tilt_y = 0;
    if has_tilt && data.len() >= 10 {
        tilt_x = data[8] as i8;
        tilt_y = data[9] as i8;
    }

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

pub struct XpPenGen2Parser;

impl ReportParser for XpPenGen2Parser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }
        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        if data[1] == 0xF0 {
            return parse_aux(data, raw, 2);
        }
        if (data[1] & 0xF0) == 0xA0 {
            return parse_gen2(data, raw);
        }
        standard_parse(data)
    }
}

pub struct XpPenDeco03Parser;

impl ReportParser for XpPenDeco03Parser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }
        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        if data[1] == 0xF0 {
            return parse_aux(data, raw, 2);
        } // Wheel parsed as aux
        if (data[1] & 0x10) != 0 {
            return parse_aux(data, raw, 2);
        }
        standard_parse(data)
    }
}

pub struct XpPenOffsetPressureParser;

impl ReportParser for XpPenOffsetPressureParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }
        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        if (data[1] & 0x10) != 0 {
            return parse_aux(data, raw, 2);
        }
        if data.len() >= 10 {
            parse_offset_pressure(data, raw, true)
        } else {
            parse_offset_pressure(data, raw, false)
        }
    }
}

pub struct XpPenOffsetAuxParser;

impl ReportParser for XpPenOffsetAuxParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }
        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        if (data[1] & 0x20) != 0 {
            return parse_aux(data, raw, 4);
        }
        standard_parse(data)
    }
}

pub struct XpPenParser;

impl ReportParser for XpPenParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 {
            return None;
        }
        let raw = data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        if (data[1] & 0x10) != 0 {
            return parse_aux(data, raw, 2);
        }
        standard_parse(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_pen_gen2() -> Result<(), Box<dyn std::error::Error>> {
        let parser = XpPenGen2Parser;
        let data: [u8; 14] = [
            0, 0xA2, 0x02, 0, 0x04, 0, 0x01, 0x00, 10, 20, 0x01, 0x03, 0, 0,
        ];
        let report = parser
            .parse(&data)
            .ok_or("XP-Pen Gen2 parser failed to parse tablet packet")?;
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 0xFFFF); // overflow u16 max clamped
        assert_eq!(report.buttons, 1);
        Ok(())
    }
}
