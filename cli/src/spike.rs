use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SpikeType {
    Page,
    Element,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    Love,
    Like,
    Meh,
    No,
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rating::Love => write!(f, "love"),
            Rating::Like => write!(f, "like"),
            Rating::Meh => write!(f, "meh"),
            Rating::No => write!(f, "no"),
        }
    }
}

impl std::str::FromStr for Rating {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "love" => Ok(Rating::Love),
            "like" => Ok(Rating::Like),
            "meh" => Ok(Rating::Meh),
            "no" => Ok(Rating::No),
            _ => Err(format!("Invalid rating: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spike {
    pub id: String,
    #[serde(rename = "type")]
    pub spike_type: SpikeType,
    pub project_key: String,
    pub page: String,
    pub url: String,
    pub reviewer: Reviewer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    pub rating: Option<Rating>,
    pub comments: String,
    pub timestamp: String,
    pub viewport: Viewport,
}

impl Spike {
    pub fn rating_str(&self) -> &str {
        match &self.rating {
            Some(r) => match r {
                Rating::Love => "love",
                Rating::Like => "like",
                Rating::Meh => "meh",
                Rating::No => "no",
            },
            None => "-",
        }
    }

    pub fn type_str(&self) -> &str {
        match self.spike_type {
            SpikeType::Page => "page",
            SpikeType::Element => "element",
        }
    }
}
