use crate::drivers::TabletData;

pub struct BambooTabletReport {
    pub x: u16,
    pub y: u16,
    pub pressure: u16,
    pub eraser: bool,
    pub near_proximity: bool,
    pub buttons: u8,
    pub aux_buttons: [bool; 4],
}

impl BambooTabletReport {
    pub fn new(report: &[u8]) -> Self {
        // Based on OTD BambooTabletReport.cs
        // X = report[2] | report[3] << 8
        // Y = report[4] | report[5] << 8
        // Pressure = bit 0 of report[1] ? (report[6] | (report[7] & 0x03) << 8) : 0
        // NearProximity = bit 7 of report[1]
        // Eraser = bit 5 of report[1]
        // buttons = bit 1, 2 of report[1]
        // aux = bit 3, 4, 5, 6 of report[7]

        let x = u16::from_le_bytes([report[2], report[3]]);
        let y = u16::from_le_bytes([report[4], report[5]]);
        
        let pressure = if (report[1] & 0x01) != 0 {
            (report[6] as u16) | (((report[7] & 0x03) as u16) << 8)
        } else {
            0
        };

        Self {
            x,
            y,
            pressure,
            eraser: (report[1] & 0x20) != 0,
            near_proximity: (report[1] & 0x80) != 0,
            buttons: (report[1] >> 1) & 0x03,
            aux_buttons: [
                (report[7] & 0x08) != 0,
                (report[7] & 0x10) != 0,
                (report[7] & 0x20) != 0,
                (report[7] & 0x40) != 0,
            ],
        }
    }
}

pub fn parse_bamboo(data: &[u8]) -> Option<TabletData> {
    if data.len() < 8 { return None; }
    
    // OTD BambooReportParser matches various IDs, but often 0x02 for pen
    match data[0] {
        0x02 => {
            let report = BambooTabletReport::new(data);
            let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            
            Some(TabletData {
                status: if report.pressure > 0 { "Contact".to_string() } else if report.near_proximity { "Hover".to_string() } else { "Out of Range".to_string() },
                x: report.x,
                y: report.y,
                pressure: report.pressure,
                tilt_x: 0,
                tilt_y: 0,
                buttons: report.buttons,
                eraser: report.eraser,
                hover_distance: 0,
                raw_data: raw,
                is_connected: true,
            })
        },
        _ => None
    }
}
