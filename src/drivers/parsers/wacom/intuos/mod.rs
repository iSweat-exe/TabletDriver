use crate::drivers::TabletData;

pub struct IntuosTabletReport {
    pub x: u16,
    pub y: u16,
    pub pressure: u16,
    pub eraser: bool,
    pub near_proximity: bool,
    pub buttons: u8,
    pub hover_distance: u8,
}

impl IntuosTabletReport {
    pub fn new(report: &[u8]) -> Self {
        // Based on OTD IntuosTabletReport.cs
        // X = report[2] | report[3] << 8
        // Y = report[4] | report[5] << 8
        // Pressure = report[6] | report[7] << 8
        // Buttons = bit 1 and 2 of report[1]
        // Eraser = bit 3 of report[1]
        // NearProximity = bit 7 of report[1]
        // HoverDistance = report[8]
        
        Self {
            x: u16::from_le_bytes([report[2], report[3]]),
            y: u16::from_le_bytes([report[4], report[5]]),
            pressure: u16::from_le_bytes([report[6], report[7]]),
            eraser: (report[1] & 0x08) != 0,
            near_proximity: (report[1] & 0x80) != 0,
            buttons: (report[1] >> 1) & 0x03,
            hover_distance: report[8],
        }
    }
}

pub fn parse_intuos(data: &[u8]) -> Option<TabletData> {
    if data.is_empty() { return None; }
    
    // Dispatch exactly like OTD IntuosReportParser.cs
    match data[0] {
        0x02 => {
            // Check if bit 6 is set (OTD: report[1].IsBitSet(6))
            if (data[1] & 0x40) != 0 {
                let report = IntuosTabletReport::new(data);
                
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
                    hover_distance: report.hover_distance,
                    raw_data: raw,
                    is_connected: true,
                })
            } else if data[1] == 0x80 {
                // OutOfRangeReport in OTD
                None
            } else {
                None
            }
        },
        _ => None
    }
}

pub fn parse_wacom_driver_intuos(data: &[u8]) -> Option<TabletData> {
    // Offset by 1 as in OTD WacomDriverIntuosReportParser.cs
    if data.len() < 2 { return None; }
    parse_intuos(&data[1..])
}
