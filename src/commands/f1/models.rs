use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct F1Calendar {
    #[serde(rename = "MRData")]
    pub mr_data: MRData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MRData {
    #[serde(rename = "RaceTable")]
    pub race_table: RaceTable,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaceTable {
    #[serde(rename = "Races")]
    pub races: Vec<Race>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Race {
    #[serde(rename = "raceName")]
    pub race_name: String,
    #[serde(rename = "Circuit")]
    pub circuit: Circuit,
    #[serde(rename = "date")]
    pub date: String,
    #[serde(rename = "time", default)]
    pub time: String,
    #[serde(rename = "round")]
    pub round: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Circuit {
    #[serde(rename = "circuitName")]
    pub circuit_name: String,
    #[serde(rename = "Location")]
    pub location: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    #[serde(rename = "locality")]
    pub locality: String,
    #[serde(rename = "country")]
    pub country: String,
} 