use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

fn parse_uclogic_aux(data: &[u8], raw: String) -> Option<TabletData> {
    if data.len() < 7 { return None; }
    Some(TabletData {
        status: "Aux".to_string(),
        buttons: data[4], // Just grab the first 8 aux buttons
        raw_data: raw,
        is_connected: true,
        ..Default::default()
    })
}

fn parse_uclogic_tablet(data: &[u8], raw: String, has_tilt: bool) -> Option<TabletData> {
    if data.len() < 8 { return None; }
    
    let x = u16::from_le_bytes([data[2], data[3]]);
    let y = u16::from_le_bytes([data[4], data[5]]);
    let pressure = u16::from_le_bytes([data[6], data[7]]);

    let mut buttons: u8 = 0;
    if (data[1] & 0x01) != 0 { buttons |= 1 << 0; }
    if (data[1] & 0x02) != 0 { buttons |= 1 << 1; }
    if (data[1] & 0x04) != 0 { buttons |= 1 << 2; }
    let eraser = (data[1] & 0x04) != 0; // standard usually puts eraser on bit 2 or sometimes 3

    let mut tilt_x = 0;
    let mut tilt_y = 0;
    if has_tilt && data.len() >= 12 {
        tilt_x = data[10] as i8;
        tilt_y = data[11] as i8;
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

pub struct UCLogicParser;

impl ReportParser for UCLogicParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 { return None; }
        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
        if data[1] == 0xC0 { return None; }
        if (data[1] & 0x40) != 0 { parse_uclogic_aux(data, raw) } else { parse_uclogic_tablet(data, raw, false) }
    }
}

pub struct UCLogicV1Parser;

impl ReportParser for UCLogicV1Parser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 { return None; }
        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
        if data[1] == 0xE0 { parse_uclogic_aux(data, raw) }
        else if (data[1] & 0x40) != 0 { parse_uclogic_tablet(data, raw, false) }
        else { None }
    }
}

pub struct UCLogicV2Parser;

impl ReportParser for UCLogicV2Parser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 { return None; }
        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
        match data[1] {
            0xE0 => parse_uclogic_aux(data, raw),
            0xF0 => None,
            _ => parse_uclogic_tablet(data, raw, true),
        }
    }
}

pub struct UCLogicTiltParser;

impl ReportParser for UCLogicTiltParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 { return None; }
        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
        if (data[1] & 0x40) != 0 { parse_uclogic_aux(data, raw) }
        else { parse_uclogic_tablet(data, raw, true) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uclogic_tablet() {
        let parser = UCLogicParser;
        let data: [u8; 8] = [0, 0x01, 0x02, 0x01, 0x04, 0x03, 0x01, 0x00];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Contact");
        assert_eq!(report.x, 258);
        assert_eq!(report.pressure, 1);
        assert_eq!(report.buttons, 1);
    }

    #[test]
    fn test_uclogic_aux() {
        let parser = UCLogicParser;
        let data: [u8; 8] = [0, 0x40, 0, 0, 5, 0, 0, 0];
        let report = parser.parse(&data).unwrap();
        assert_eq!(report.status, "Aux");
        assert_eq!(report.buttons, 5);
    }
}
