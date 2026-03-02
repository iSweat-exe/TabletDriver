use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TabletConfiguration {
    pub name: String,
    pub specifications: Specifications,
    pub digitizer_identifiers: Vec<DigitizerIdentifier>,
    pub attributes: Option<Attributes>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Specifications {
    pub digitizer: DigitizerSpecs,
    pub pen: PenSpecs,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DigitizerSpecs {
    pub width: f32,
    pub height: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PenSpecs {
    pub max_pressure: u16,
    pub button_count: Option<u8>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DigitizerIdentifier {
    #[serde(rename = "VendorID")]
    pub vendor_id: u16,
    #[serde(rename = "ProductID")]
    pub product_id: u16,
    pub input_report_length: Option<usize>,
    pub output_report_length: Option<usize>,
    pub report_parser: String,
    pub output_init_report: Option<Vec<String>>,
    pub feature_init_report: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Attributes {
    pub libinputoverride: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_g640() {
        let json = r#"{
  "Name": "XP-Pen Star G640 (Variant 2)",
  "Specifications": {
    "Digitizer": {
      "Width": 159.99,
      "Height": 99.99,
      "MaxX": 15999,
      "MaxY": 9999
    },
    "Pen": {
      "MaxPressure": 8191,
      "ButtonCount": 2
    }
  },
  "DigitizerIdentifiers": [
    {
      "VendorID": 10429,
      "ProductID": 2324,
      "InputReportLength": 12,
      "OutputReportLength": 12,
      "ReportParser": "OpenTabletDriver.Configurations.Parsers.XP_Pen.XP_PenReportParser",
      "OutputInitReport": [
        "ArAE"
      ]
    }
  ],
  "Attributes": {
    "libinputoverride": "1"
  }
}"#;

        let config: TabletConfiguration = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "XP-Pen Star G640 (Variant 2)");
        assert_eq!(config.digitizer_identifiers[0].vendor_id, 10429);
        assert_eq!(config.specifications.digitizer.width, 159.99);
    }

    #[test]
    fn test_deserialize_ctl472() {
        let json = r#"{
  "Name": "Wacom CTL-472",
  "Specifications": {
    "Digitizer": {
      "Width": 152,
      "Height": 95,
      "MaxX": 15200,
      "MaxY": 9500
    },
    "Pen": {
      "MaxPressure": 2047,
      "ButtonCount": 2
    }
  },
  "DigitizerIdentifiers": [
    {
      "VendorID": 1386,
      "ProductID": 890,
      "InputReportLength": 10,
      "ReportParser": "OpenTabletDriver.Configurations.Parsers.Wacom.Intuos.IntuosReportParser",
      "FeatureInitReport": [
        "AgI="
      ]
    }
  ],
  "Attributes": {}
}"#;

        let config: TabletConfiguration = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "Wacom CTL-472");
        assert_eq!(
            config.digitizer_identifiers[0]
                .feature_init_report
                .as_ref()
                .unwrap()[0],
            "AgI="
        );
    }
}
