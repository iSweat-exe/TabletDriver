use super::intuos_v1::IntuosV1Parser;
use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

// --- Intuos 4 ---

pub struct Intuos4Parser {
    inner_v1: IntuosV1Parser,
}

impl Intuos4Parser {
    pub fn new() -> Self {
        Self {
            inner_v1: IntuosV1Parser::new(),
        }
    }

    fn parse_internal(&self, data: &[u8], raw: String) -> Option<TabletData> {
        match data[0] {
            0x02 => match data[1] {
                0xEC | 0xAC => self.parse_mouse(data, raw),
                _ => self.inner_v1.parse_internal(data, raw),
            },
            0x10 => self.inner_v1.parse_internal(data, raw),
            0x0C => self.parse_aux(data, raw),
            _ => None,
        }
    }

    fn parse_mouse(&self, data: &[u8], raw: String) -> Option<TabletData> {
        if data.len() < 10 {
            return None;
        }
        let x = (((data[2] as u16) << 8) | (data[3] as u16)) << 1 | (((data[9] >> 1) & 1) as u16);
        let y = (((data[4] as u16) << 8) | (data[5] as u16)) << 1 | ((data[9] & 1) as u16);
        let mut buttons: u8 = 0;
        if (data[6] & 0x01) != 0 {
            buttons |= 1 << 0;
        }
        if (data[6] & 0x04) != 0 {
            buttons |= 1 << 1;
        }
        if (data[6] & 0x02) != 0 {
            buttons |= 1 << 2;
        }
        if (data[6] & 0x08) != 0 {
            buttons |= 1 << 3;
        }
        if (data[6] & 0x10) != 0 {
            buttons |= 1 << 4;
        }

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

    fn parse_aux(&self, data: &[u8], raw: String) -> Option<TabletData> {
        if data.len() < 4 {
            return None;
        }
        let buttons = data[3];
        Some(TabletData {
            status: "Aux".to_string(),
            buttons,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }
}

impl ReportParser for Intuos4Parser {
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

pub struct WacomDriverIntuos4Parser {
    inner: Intuos4Parser,
}

impl WacomDriverIntuos4Parser {
    pub fn new() -> Self {
        Self {
            inner: Intuos4Parser::new(),
        }
    }
}

impl ReportParser for WacomDriverIntuos4Parser {
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
