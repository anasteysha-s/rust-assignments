// part_2.rs - Updated with proper types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Represents the response type
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Success,
    Error,
}

/// Public tariff information
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct PublicTariff {
    pub id: u32,
    pub price: u32,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub description: String,
}

/// Private tariff information
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct PrivateTariff {
    pub client_price: u32,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub description: String,
}

/// Stream settings and configuration
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Stream {
    pub user_id: String, // UUID as string (could use uuid crate for stricter typing)
    pub is_private: bool,
    pub settings: u32,
    pub shard_url: Url,
    pub public_tariff: PublicTariff,
    pub private_tariff: PrivateTariff,
}

/// Gift item
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Gift {
    pub id: u32,
    pub price: u32,
    pub description: String,
}

/// Debug information
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Debug {
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub at: DateTime<Utc>,
}

/// Main request structure
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Request {
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    pub stream: Stream,
    pub gifts: Vec<Gift>,
    pub debug: Debug,
}

impl Request {
    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Serialize to JSON string (for testing)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Main function to demonstrate the conversion
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let json_data = r#"{
  "type": "success",
  "stream": {
    "user_id": "8d234120-0bda-49b2-b7e0-fbd3912f6cbf",
    "is_private": false,
    "settings": 45345,
    "shard_url": "https://n3.example.com/sapi",
    "public_tariff": {
      "id": 1,
      "price": 100,
      "duration": "1h",
      "description": "test public tariff"
    },
    "private_tariff": {
      "client_price": 250,
      "duration": "1m",
      "description": "test private tariff"
    }
  },
  "gifts": [{
    "id": 1,
    "price": 2,
    "description": "Gift 1"
  }, {
    "id": 2,
    "price": 3,
    "description": "Gift 2"
  }],
  "debug": {
    "duration": "234ms",
    "at": "2019-06-28T08:35:46+00:00"
  }
}"#;

    println!("=== Original JSON ===");
    println!("{}\n", json_data);

    // Deserialize from JSON
    let request = Request::from_json(json_data)?;
    println!("=== Deserialized Request ===");
    println!("{:#?}\n", request);

    // Serialize to TOML
    let toml_output = request.to_toml()?;
    println!("=== TOML Output ===");
    println!("{}", toml_output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JSON: &str = r#"{
  "type": "success",
  "stream": {
    "user_id": "8d234120-0bda-49b2-b7e0-fbd3912f6cbf",
    "is_private": false,
    "settings": 45345,
    "shard_url": "https://n3.example.com/sapi",
    "public_tariff": {
      "id": 1,
      "price": 100,
      "duration": "1h",
      "description": "test public tariff"
    },
    "private_tariff": {
      "client_price": 250,
      "duration": "1m",
      "description": "test private tariff"
    }
  },
  "gifts": [{
    "id": 1,
    "price": 2,
    "description": "Gift 1"
  }, {
    "id": 2,
    "price": 3,
    "description": "Gift 2"
  }],
  "debug": {
    "duration": "234ms",
    "at": "2019-06-28T08:35:46+00:00"
  }
}"#;

    #[test]
    fn test_deserialize_json() {
        let request = Request::from_json(TEST_JSON);
        assert!(request.is_ok());

        let request = request.unwrap();
        assert_eq!(request.response_type, ResponseType::Success);
        assert_eq!(
            request.stream.user_id,
            "8d234120-0bda-49b2-b7e0-fbd3912f6cbf"
        );
        assert!(!request.stream.is_private);
        assert_eq!(request.stream.settings, 45345);
    }

    #[test]
    fn test_shard_url_is_valid() {
        let request = Request::from_json(TEST_JSON).unwrap();
        assert_eq!(request.stream.shard_url.scheme(), "https");
        assert_eq!(request.stream.shard_url.host_str(), Some("n3.example.com"));
        assert_eq!(request.stream.shard_url.path(), "/sapi");
    }

    #[test]
    fn test_public_tariff_duration() {
        let request = Request::from_json(TEST_JSON).unwrap();
        let public_tariff = &request.stream.public_tariff;

        assert_eq!(public_tariff.id, 1);
        assert_eq!(public_tariff.price, 100);
        assert_eq!(public_tariff.duration, Duration::from_secs(3600)); // 1 hour
        assert_eq!(public_tariff.description, "test public tariff");
    }

    #[test]
    fn test_private_tariff_duration() {
        let request = Request::from_json(TEST_JSON).unwrap();
        let private_tariff = &request.stream.private_tariff;

        assert_eq!(private_tariff.client_price, 250);
        assert_eq!(private_tariff.duration, Duration::from_secs(60)); // 1 minute
        assert_eq!(private_tariff.description, "test private tariff");
    }

    #[test]
    fn test_gifts() {
        let request = Request::from_json(TEST_JSON).unwrap();
        assert_eq!(request.gifts.len(), 2);

        assert_eq!(request.gifts[0].id, 1);
        assert_eq!(request.gifts[0].price, 2);
        assert_eq!(request.gifts[0].description, "Gift 1");

        assert_eq!(request.gifts[1].id, 2);
        assert_eq!(request.gifts[1].price, 3);
        assert_eq!(request.gifts[1].description, "Gift 2");
    }

    #[test]
    fn test_debug_info_duration() {
        let request = Request::from_json(TEST_JSON).unwrap();
        assert_eq!(request.debug.duration, Duration::from_millis(234));
    }

    #[test]
    fn test_debug_info_datetime() {
        let request = Request::from_json(TEST_JSON).unwrap();
        
        // Parse the expected datetime
        let expected = DateTime::parse_from_rfc3339("2019-06-28T08:35:46+00:00")
            .unwrap()
            .with_timezone(&Utc);
        
        assert_eq!(request.debug.at, expected);
    }

    #[test]
    fn test_serialize_to_toml() {
        let request = Request::from_json(TEST_JSON).unwrap();
        let toml_result = request.to_toml();

        assert!(toml_result.is_ok());
        let toml_str = toml_result.unwrap();

        // Verify TOML contains key sections
        assert!(toml_str.contains("type") || toml_str.contains("\"type\""));
        assert!(toml_str.contains("[stream]"));
        assert!(toml_str.contains("[[gifts]]"));
        assert!(toml_str.contains("[debug]"));
    }

    #[test]
    fn test_roundtrip_json() {
        let original = Request::from_json(TEST_JSON).unwrap();
        let json_str = original.to_json().unwrap();
        let roundtrip = Request::from_json(&json_str).unwrap();

        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_toml_deserialize_back() {
        let original = Request::from_json(TEST_JSON).unwrap();
        let toml_str = original.to_toml().unwrap();

        // Deserialize back from TOML
        let from_toml: Result<Request, toml::de::Error> = toml::from_str(&toml_str);
        assert!(from_toml.is_ok());

        let roundtrip = from_toml.unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_response_type_enum() {
        let success_json = r#"{"type": "success", "stream": {"user_id": "test", "is_private": false, "settings": 1, "shard_url": "https://example.com", "public_tariff": {"id": 1, "price": 1, "duration": "1h", "description": "d"}, "private_tariff": {"client_price": 1, "duration": "1m", "description": "d"}}, "gifts": [], "debug": {"duration": "1ms", "at": "2019-06-28T08:35:46+00:00"}}"#;
        let request = Request::from_json(success_json).unwrap();
        assert_eq!(request.response_type, ResponseType::Success);
    }

    #[test]
    fn test_empty_gifts() {
        let json_with_empty_gifts = r#"{
            "type": "success",
            "stream": {
                "user_id": "test-id",
                "is_private": true,
                "settings": 1,
                "shard_url": "https://example.com",
                "public_tariff": {
                    "id": 1,
                    "price": 50,
                    "duration": "30m",
                    "description": "test"
                },
                "private_tariff": {
                    "client_price": 100,
                    "duration": "1h",
                    "description": "test"
                }
            },
            "gifts": [],
            "debug": {
                "duration": "100ms",
                "at": "2019-06-28T08:35:46+00:00"
            }
        }"#;

        let request = Request::from_json(json_with_empty_gifts).unwrap();
        assert_eq!(request.gifts.len(), 0);
    }

    #[test]
    fn test_duration_formats() {
        // Test various duration formats supported by humantime_serde
        let json_1h = r#"{"type": "success", "stream": {"user_id": "test", "is_private": false, "settings": 1, "shard_url": "https://example.com", "public_tariff": {"id": 1, "price": 1, "duration": "1h", "description": "d"}, "private_tariff": {"client_price": 1, "duration": "60s", "description": "d"}}, "gifts": [], "debug": {"duration": "500ms", "at": "2019-06-28T08:35:46+00:00"}}"#;
        
        let request = Request::from_json(json_1h).unwrap();
        assert_eq!(request.stream.public_tariff.duration, Duration::from_secs(3600));
        assert_eq!(request.stream.private_tariff.duration, Duration::from_secs(60));
        assert_eq!(request.debug.duration, Duration::from_millis(500));
    }

    #[test]
    fn test_url_manipulation() {
        let request = Request::from_json(TEST_JSON).unwrap();
        let url = &request.stream.shard_url;
        
        // Test URL components
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("n3.example.com"));
        assert_eq!(url.port(), None); // Default HTTPS port
        assert_eq!(url.path(), "/sapi");
    }
}