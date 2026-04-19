use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct BambooV2AuxParser;

impl ReportParser for BambooV2AuxParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.is_empty() { return None; }
        let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");

        if data[0] == 0x02 {
             // IntuosV2AuxReport style
             if data.len() < 2 { return None; }
             let buttons = data[1];
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
