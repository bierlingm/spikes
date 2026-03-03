use serde::{Deserialize, Serialize};

/// Generic paginated response from the API
/// The worker returns { data: [...items], next_cursor: string|null }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
}

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
    pub viewport: Option<Viewport>,
    /// Whether this spike has been resolved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<bool>,
    /// ISO 8601 timestamp when spike was resolved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
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

    /// Check if this spike is resolved
    pub fn is_resolved(&self) -> bool {
        self.resolved.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rating_from_str() {
        assert_eq!("love".parse::<Rating>().unwrap(), Rating::Love);
        assert_eq!("LIKE".parse::<Rating>().unwrap(), Rating::Like);
        assert_eq!("Meh".parse::<Rating>().unwrap(), Rating::Meh);
        assert_eq!("no".parse::<Rating>().unwrap(), Rating::No);
        assert!("invalid".parse::<Rating>().is_err());
    }

    #[test]
    fn test_rating_display() {
        assert_eq!(format!("{}", Rating::Love), "love");
        assert_eq!(format!("{}", Rating::Like), "like");
        assert_eq!(format!("{}", Rating::Meh), "meh");
        assert_eq!(format!("{}", Rating::No), "no");
    }

    #[test]
    fn test_spike_type_serialization() {
        let page = SpikeType::Page;
        let element = SpikeType::Element;

        assert_eq!(serde_json::to_string(&page).unwrap(), "\"page\"");
        assert_eq!(serde_json::to_string(&element).unwrap(), "\"element\"");
    }

    #[test]
    fn test_spike_type_deserialization() {
        let page: SpikeType = serde_json::from_str("\"page\"").unwrap();
        let element: SpikeType = serde_json::from_str("\"element\"").unwrap();

        assert_eq!(page, SpikeType::Page);
        assert_eq!(element, SpikeType::Element);
    }

    #[test]
    fn test_rating_serialization() {
        assert_eq!(serde_json::to_string(&Rating::Love).unwrap(), "\"love\"");
        assert_eq!(serde_json::to_string(&Rating::Like).unwrap(), "\"like\"");
        assert_eq!(serde_json::to_string(&Rating::Meh).unwrap(), "\"meh\"");
        assert_eq!(serde_json::to_string(&Rating::No).unwrap(), "\"no\"");
    }

    #[test]
    fn test_rating_deserialization() {
        let love: Rating = serde_json::from_str("\"love\"").unwrap();
        let like: Rating = serde_json::from_str("\"like\"").unwrap();
        let meh: Rating = serde_json::from_str("\"meh\"").unwrap();
        let no: Rating = serde_json::from_str("\"no\"").unwrap();

        assert_eq!(love, Rating::Love);
        assert_eq!(like, Rating::Like);
        assert_eq!(meh, Rating::Meh);
        assert_eq!(no, Rating::No);
    }

    #[test]
    fn test_spike_deserialization() {
        let json = r#"{
            "id": "test-id-123",
            "type": "page",
            "projectKey": "my-project",
            "page": "index.html",
            "url": "http://localhost:3000/index.html",
            "reviewer": {"id": "r1", "name": "Alice"},
            "rating": "like",
            "comments": "Great work!",
            "timestamp": "2024-01-15T10:30:00Z",
            "viewport": {"width": 1920, "height": 1080}
        }"#;

        let spike: Spike = serde_json::from_str(json).unwrap();

        assert_eq!(spike.id, "test-id-123");
        assert_eq!(spike.spike_type, SpikeType::Page);
        assert_eq!(spike.project_key, "my-project");
        assert_eq!(spike.page, "index.html");
        assert_eq!(spike.reviewer.id, "r1");
        assert_eq!(spike.reviewer.name, "Alice");
        assert_eq!(spike.rating, Some(Rating::Like));
        assert_eq!(spike.comments, "Great work!");
        assert!(spike.selector.is_none());
        assert!(spike.element_text.is_none());
        assert!(spike.bounding_box.is_none());
    }

    #[test]
    fn test_element_spike_deserialization() {
        let json = concat!(
            "{",
            "\"id\": \"elem-456\",",
            "\"type\": \"element\",",
            "\"projectKey\": \"my-project\",",
            "\"page\": \"index.html\",",
            "\"url\": \"http://localhost:3000/index.html\",",
            "\"reviewer\": {\"id\": \"r1\", \"name\": \"Bob\"},",
            "\"selector\": \".hero-title\",",
            "\"elementText\": \"Welcome\",",
            "\"boundingBox\": {\"x\": 100.0, \"y\": 50.0, \"width\": 200.0, \"height\": 40.0},",
            "\"rating\": \"love\",",
            "\"comments\": \"Love this headline!\",",
            "\"timestamp\": \"2024-01-15T10:31:00Z\",",
            "\"viewport\": {\"width\": 1920, \"height\": 1080}",
            "}"
        );

        let spike: Spike = serde_json::from_str(json).unwrap();

        assert_eq!(spike.spike_type, SpikeType::Element);
        assert_eq!(spike.selector, Some(".hero-title".to_string()));
        assert_eq!(spike.element_text, Some("Welcome".to_string()));
        assert!(spike.bounding_box.is_some());

        let bbox = spike.bounding_box.unwrap();
        assert_eq!(bbox.x, 100.0);
        assert_eq!(bbox.y, 50.0);
        assert_eq!(bbox.width, 200.0);
        assert_eq!(bbox.height, 40.0);
    }

    #[test]
    fn test_spike_serialization_roundtrip() {
        let spike = Spike {
            id: "test-789".to_string(),
            spike_type: SpikeType::Page,
            project_key: "test-project".to_string(),
            page: "about.html".to_string(),
            url: "http://localhost:3000/about.html".to_string(),
            reviewer: Reviewer {
                id: "reviewer-1".to_string(),
                name: "Charlie".to_string(),
            },
            selector: None,
            element_text: None,
            bounding_box: None,
            rating: Some(Rating::Meh),
            comments: "Could be better".to_string(),
            timestamp: "2024-01-15T11:00:00Z".to_string(),
            viewport: Some(Viewport {
                width: 1280,
                height: 720,
            }),
            resolved: None,
            resolved_at: None,
        };

        let json = serde_json::to_string(&spike).unwrap();
        let deserialized: Spike = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, spike.id);
        assert_eq!(deserialized.spike_type, spike.spike_type);
        assert_eq!(deserialized.project_key, spike.project_key);
        assert_eq!(deserialized.rating, spike.rating);
    }

    #[test]
    fn test_spike_rating_str() {
        let mut spike = Spike {
            id: "test".to_string(),
            spike_type: SpikeType::Page,
            project_key: "p".to_string(),
            page: "page".to_string(),
            url: "url".to_string(),
            reviewer: Reviewer { id: "r".to_string(), name: "R".to_string() },
            selector: None,
            element_text: None,
            bounding_box: None,
            rating: Some(Rating::Love),
            comments: "".to_string(),
            timestamp: "".to_string(),
            viewport: None,
            resolved: None,
            resolved_at: None,
        };

        assert_eq!(spike.rating_str(), "love");
        spike.rating = Some(Rating::No);
        assert_eq!(spike.rating_str(), "no");
        spike.rating = None;
        assert_eq!(spike.rating_str(), "-");
    }

    #[test]
    fn test_spike_type_str() {
        let mut spike = Spike {
            id: "test".to_string(),
            spike_type: SpikeType::Page,
            project_key: "p".to_string(),
            page: "page".to_string(),
            url: "url".to_string(),
            reviewer: Reviewer { id: "r".to_string(), name: "R".to_string() },
            selector: None,
            element_text: None,
            bounding_box: None,
            rating: None,
            comments: "".to_string(),
            timestamp: "".to_string(),
            viewport: None,
            resolved: None,
            resolved_at: None,
        };

        assert_eq!(spike.type_str(), "page");
        spike.spike_type = SpikeType::Element;
        assert_eq!(spike.type_str(), "element");
    }

    #[test]
    fn test_spike_null_rating() {
        // Spikes can have null rating
        let json = r#"{
            "id": "no-rating",
            "type": "page",
            "projectKey": "proj",
            "page": "page.html",
            "url": "http://example.com",
            "reviewer": {"id": "r1", "name": "Test"},
            "rating": null,
            "comments": "Just a comment",
            "timestamp": "2024-01-15T12:00:00Z"
        }"#;

        let spike: Spike = serde_json::from_str(json).unwrap();
        assert!(spike.rating.is_none());
    }

    #[test]
    fn test_spike_missing_optional_fields() {
        // Optional fields can be missing entirely
        let json = r#"{
            "id": "minimal",
            "type": "page",
            "projectKey": "proj",
            "page": "page.html",
            "url": "http://example.com",
            "reviewer": {"id": "r1", "name": "Test"},
            "comments": "A comment",
            "timestamp": "2024-01-15T12:00:00Z"
        }"#;

        let spike: Spike = serde_json::from_str(json).unwrap();
        assert!(spike.rating.is_none());
        assert!(spike.selector.is_none());
        assert!(spike.element_text.is_none());
        assert!(spike.bounding_box.is_none());
        assert!(spike.viewport.is_none());
        assert!(spike.resolved.is_none());
        assert!(spike.resolved_at.is_none());
        assert!(!spike.is_resolved());
    }

    #[test]
    fn test_spike_resolved() {
        let json = r#"{
            "id": "resolved-spike",
            "type": "page",
            "projectKey": "proj",
            "page": "page.html",
            "url": "http://example.com",
            "reviewer": {"id": "r1", "name": "Test"},
            "comments": "A comment",
            "timestamp": "2024-01-15T12:00:00Z",
            "resolved": true,
            "resolvedAt": "2024-01-16T10:00:00Z"
        }"#;

        let spike: Spike = serde_json::from_str(json).unwrap();
        assert_eq!(spike.resolved, Some(true));
        assert_eq!(spike.resolved_at, Some("2024-01-16T10:00:00Z".to_string()));
        assert!(spike.is_resolved());
    }

    #[test]
    fn test_spike_resolved_false() {
        let json = r#"{
            "id": "unresolved-spike",
            "type": "page",
            "projectKey": "proj",
            "page": "page.html",
            "url": "http://example.com",
            "reviewer": {"id": "r1", "name": "Test"},
            "comments": "A comment",
            "timestamp": "2024-01-15T12:00:00Z",
            "resolved": false
        }"#;

        let spike: Spike = serde_json::from_str(json).unwrap();
        assert_eq!(spike.resolved, Some(false));
        assert!(spike.resolved_at.is_none());
        assert!(!spike.is_resolved());
    }

    #[test]
    fn test_paginated_response_with_spikes() {
        let json = r#"{
            "data": [
                {
                    "id": "spike-1",
                    "type": "page",
                    "projectKey": "proj",
                    "page": "index.html",
                    "url": "http://example.com",
                    "reviewer": {"id": "r1", "name": "Alice"},
                    "comments": "First spike",
                    "timestamp": "2024-01-15T10:00:00Z"
                },
                {
                    "id": "spike-2",
                    "type": "element",
                    "projectKey": "proj",
                    "page": "index.html",
                    "url": "http://example.com",
                    "reviewer": {"id": "r2", "name": "Bob"},
                    "selector": ".btn",
                    "comments": "Second spike",
                    "timestamp": "2024-01-15T11:00:00Z"
                }
            ],
            "next_cursor": "2024-01-15T11:00:00Z"
        }"#;

        let response: PaginatedResponse<Spike> = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].id, "spike-1");
        assert_eq!(response.data[1].id, "spike-2");
        assert_eq!(response.next_cursor, Some("2024-01-15T11:00:00Z".to_string()));
    }

    #[test]
    fn test_paginated_response_null_cursor() {
        let json = r#"{
            "data": [
                {
                    "id": "spike-last",
                    "type": "page",
                    "projectKey": "proj",
                    "page": "index.html",
                    "url": "http://example.com",
                    "reviewer": {"id": "r1", "name": "Alice"},
                    "comments": "Last spike",
                    "timestamp": "2024-01-15T12:00:00Z"
                }
            ],
            "next_cursor": null
        }"#;

        let response: PaginatedResponse<Spike> = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].id, "spike-last");
        assert!(response.next_cursor.is_none());
    }

    #[test]
    fn test_paginated_response_empty_data() {
        let json = r#"{
            "data": [],
            "next_cursor": null
        }"#;

        let response: PaginatedResponse<Spike> = serde_json::from_str(json).unwrap();
        assert!(response.data.is_empty());
        assert!(response.next_cursor.is_none());
    }

    #[test]
    fn test_paginated_response_missing_cursor() {
        // next_cursor is optional, can be missing
        let json = r#"{
            "data": [
                {
                    "id": "spike-x",
                    "type": "page",
                    "projectKey": "proj",
                    "page": "page.html",
                    "url": "http://example.com",
                    "reviewer": {"id": "r1", "name": "Test"},
                    "comments": "Comment",
                    "timestamp": "2024-01-15T13:00:00Z"
                }
            ]
        }"#;

        let response: PaginatedResponse<Spike> = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        // next_cursor should be None when missing
        assert!(response.next_cursor.is_none());
    }
}
