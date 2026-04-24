use super::intuos_v1::IntuosV1Parser;
use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

// Intuos 3

pub struct Intuos3Parser {
    inner_v1: IntuosV1Parser,
}

impl Intuos3Parser {
    pub fn new() -> Self {
        Self {
            inner_v1: IntuosV1Parser::new(),
        }
    }
}

impl Default for Intuos3Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Intuos3Parser {
    pub(crate) fn parse_internal(&self, data: &[u8], raw: String) -> Option<TabletData> {
        match data[0] {
            0x02 => match data[1] {
                0xF0..=0xFF | 0xB0..=0xBF => self.parse_mouse(data, raw),
                _ => self.inner_v1.parse_internal(data, raw),
            },
            0x10 => self.inner_v1.parse_internal(data, raw),
            0x03 => self.inner_v1.parse_aux(data, raw),
            0x0C => self.parse_aux(data, raw, false),
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
        if (data[8] & 0x04) != 0 {
            buttons |= 1 << 0;
        }
        if (data[8] & 0x10) != 0 {
            buttons |= 1 << 1;
        }
        if (data[8] & 0x08) != 0 {
            buttons |= 1 << 2;
        }
        if (data[8] & 0x20) != 0 {
            buttons |= 1 << 3;
        }
        if (data[8] & 0x40) != 0 {
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

    pub(crate) fn parse_aux(&self, data: &[u8], raw: String, extra: bool) -> Option<TabletData> {
        if data.len() < 7 {
            return None;
        }
        let mut buttons: u16 = 0;
        let b5 = data[5];
        let b6 = data[6];

        if (b5 & 1) != 0 {
            buttons |= 1 << 0;
        }
        if (b5 & 2) != 0 {
            buttons |= 1 << 1;
        }
        if (b5 & 4) != 0 {
            buttons |= 1 << 2;
        }
        if (b5 & 8) != 0 {
            buttons |= 1 << 3;
        }
        if (b6 & 1) != 0 {
            buttons |= 1 << 4;
        }
        if (b6 & 2) != 0 {
            buttons |= 1 << 5;
        }
        if (b6 & 4) != 0 {
            buttons |= 1 << 6;
        }
        if (b6 & 8) != 0 {
            buttons |= 1 << 7;
        }

        if extra {
            if (b5 & 16) != 0 {
                buttons |= 1 << 8;
            }
            if (b6 & 16) != 0 {
                buttons |= 1 << 9;
            }
        }

        Some(TabletData {
            status: "Aux".to_string(),
            buttons: buttons as u8,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }
}

impl ReportParser for Intuos3Parser {
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

pub struct Intuos3ExtraAuxParser {
    inner: Intuos3Parser,
}

impl Intuos3ExtraAuxParser {
    pub fn new() -> Self {
        Self {
            inner: Intuos3Parser::new(),
        }
    }
}

impl Default for Intuos3ExtraAuxParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportParser for Intuos3ExtraAuxParser {
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
            0x0C => self.inner.parse_aux(data, raw, true),
            _ => self.inner.parse_internal(data, raw),
        }
    }
}

pub struct WacomDriverIntuos3Parser {
    inner: Intuos3Parser,
}

impl WacomDriverIntuos3Parser {
    pub fn new() -> Self {
        Self {
            inner: Intuos3Parser::new(),
        }
    }
}

impl Default for WacomDriverIntuos3Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportParser for WacomDriverIntuos3Parser {
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
