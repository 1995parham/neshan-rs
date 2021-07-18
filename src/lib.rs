use reqwest::header;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Neshan client.
/// https://platform.neshan.org/api/getting-started
pub struct Client {
    client: reqwest::Client,
}

pub struct Point {
    pub longitude: f64,
    pub latitude: f64,
}

pub enum Type {
    Car,
    Motorcycle,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Routes {
    pub routes: Vec<Route>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    pub legs: Vec<Leg>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Leg {
    pub summary: String,
    pub duration: Duration,
    pub distance: Distance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Distance {
    pub value: u64,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Duration {
    pub value: u64,
    pub text: String,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Car => f.write_str("car"),
            Type::Motorcycle => f.write_str("motorcycle"),
        }
    }
}

impl Client {
    /// create client for communicating with neshan.
    pub fn new(api_key: &str) -> Client {
        let mut headers = header::HeaderMap::new();
        headers.insert("Api-Key", header::HeaderValue::from_str(api_key).unwrap());

        let client = reqwest::Client::builder()
            .user_agent("khers")
            .default_headers(headers)
            .build()
            .unwrap();

        Client { client }
    }

    /// route finds route(s) from origin to destination.
    ///
    /// avoid_traffic_zone finds route(s) that doesn't cross the traffic zone.
    /// avoid_odd_even_zone finds route(s) that doesn's cross the odd_even_zone.
    /// alternative_paths returns alternative routes besides the primary route.
    pub async fn route(
        &self,
        vehicle: Type,
        origin: Point,
        destination: Point,
        avoid_traffic_zone: bool,
        avoid_odd_even_zone: bool,
        alternative_paths: bool,
    ) -> Result<Routes, Box<dyn std::error::Error>> {
        let res = self
            .client
            .get("https://api.neshan.org/v3/direction")
            .query(&[
                ("type", vehicle.to_string()),
                (
                    "origin",
                    format!("{},{}", origin.latitude, origin.longitude),
                ),
                (
                    "destination",
                    format!("{},{}", destination.latitude, destination.longitude),
                ),
                ("avoid_traffic_zone", avoid_traffic_zone.to_string()),
                ("avoid_odd_event_zone", avoid_odd_even_zone.to_string()),
                ("alternative", alternative_paths.to_string()),
            ])
            .send()
            .await?
            .error_for_status()?;

        let routes = res.json::<Routes>().await?;

        Ok(routes)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn request() {
        let client = super::Client::new("service.jd3J9YLeP2OWuRYPPbXlqgdoMqEIm38G5DaN50WN");
        let routes = client
            .route(
                super::Type::Car,
                super::Point {
                    latitude: 35.731984409609694,
                    longitude: 51.392684661470156,
                },
                super::Point {
                    latitude: 35.723680037006304,
                    longitude: 50.953103738230396,
                },
                true,
                true,
                false,
            )
            .await
            .unwrap();

        println!("{:?}", routes);
    }
}
