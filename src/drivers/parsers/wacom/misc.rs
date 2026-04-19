use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct Wacom64bAuxParser;

impl ReportParser for Wacom64bAuxParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 { return None; }
        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");

        if data[2] == 0x81 {
            // Touch mask report, usually translates to "proximity out" for all fingers
            return None; 
        }

        let n_chunks = data[1] as usize;
        let mut buttons: u8 = 0;

        for i in 0..n_chunks {
            let offset = (i << 3) + 2;
            if offset + 1 >= data.len() { break; }
            let id = data[offset];
            if id == 0x80 {
                // Aux Byte
                let aux_byte = data[offset + 1];
                if (aux_byte & 0x01) != 0 { buttons |= 1 << 0; }
                if (aux_byte & 0x02) != 0 { buttons |= 1 << 1; }
                if (aux_byte & 0x04) != 0 { buttons |= 1 << 2; }
                if (aux_byte & 0x08) != 0 { buttons |= 1 << 3; }
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
}
