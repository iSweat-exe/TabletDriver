use super::intuos_v1::IntuosV1Parser;
use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

// --- Intuos Pro ---

pub struct IntuosProParser {
    inner_v1: IntuosV1Parser,
}

impl IntuosProParser {
    pub fn new() -> Self {
        Self {
            inner_v1: IntuosV1Parser::new(),
        }
    }

    fn parse_internal(&self, data: &[u8], raw: String) -> Option<TabletData> {
        match data[0] {
            0x02 | 0x10 => self.inner_v1.parse_internal(data, raw),
            0x03 => self.parse_aux(data, raw),
            _ => None,
        }
    }

    fn parse_aux(&self, data: &[u8], raw: String) -> Option<TabletData> {
        if data.len() < 5 {
            return None;
        }
        let buttons = data[4];
        Some(TabletData {
            status: "Aux".to_string(),
            buttons,
            raw_data: raw,
            is_connected: true,
            ..Default::default()
        })
    }
}

impl ReportParser for IntuosProParser {
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

pub struct WacomDriverIntuosProParser {
    inner: IntuosProParser,
}

impl WacomDriverIntuosProParser {
    pub fn new() -> Self {
        Self {
            inner: IntuosProParser::new(),
        }
    }
}

impl ReportParser for WacomDriverIntuosProParser {
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
