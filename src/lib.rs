use reqwest::header;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Neshan client based on its api documentation.
/// <https://platform.neshan.org/api/getting-started>
pub struct Client {
    client: reqwest::Client,
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    code: i32,
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AppError {{ code: {}, message: {} }}",
            self.code, self.message
        )
    }
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
pub struct PostalAddress {
    pub formatted_address: String,
    pub route_name: String,
    pub neighbourhood: Option<String>,
    pub city: String,
    pub state: String,
    pub place: Option<String>,
    pub municipality_zone: Option<String>,
    pub in_traffic_zone: bool,
    pub in_odd_even_zone: bool,
}

/// distance from origin to destination in persian text form and meter.
#[derive(Debug, Serialize, Deserialize)]
pub struct Distance {
    pub value: f64,
    pub text: String,
}

/// distance from origin to destination in persian text form and seconds.
#[derive(Debug, Serialize, Deserialize)]
pub struct Duration {
    pub value: f64,
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
            .user_agent("neshan-rs")
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
            .await?;

        if !res.status().is_success() {
            let err = res.json::<Error>().await?;

            return Err(Box::new(err));
        }

        let routes = res.json::<Routes>().await?;

        Ok(routes)
    }

    /// find postal address for the given point.
    /// https://platform.neshan.org/api/reverse-geocoding
    pub async fn reverse_geocode(
        &self,
        point: Point,
    ) -> Result<PostalAddress, Box<dyn std::error::Error>> {
        let res = self
            .client
            .get("https://api.neshan.org/v2/reverse")
            .query(&[
                ("lat", point.latitude.to_string()),
                ("lng", point.longitude.to_string()),
            ])
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.json::<Error>().await?;

            return Err(Box::new(err));
        }

        let postal_address = res.json::<PostalAddress>().await?;

        Ok(postal_address)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn routes() {
        let api_key = std::env::var("NESHAN_RS_API_KEY").unwrap();

        let client = super::Client::new(&api_key);
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

    #[tokio::test]
    async fn reverse_geocode() {
        let api_key = std::env::var("NESHAN_RS_API_KEY").unwrap();

        let client = super::Client::new(&api_key);
        let postal_address = client
            .reverse_geocode(super::Point {
                latitude: 35.731984409609694,
                longitude: 51.392684661470156,
            })
            .await
            .unwrap();

        assert_eq!(postal_address.neighbourhood.as_ref().unwrap(), "قزل قلعه");
        assert_eq!(postal_address.municipality_zone.as_ref().unwrap(), "6");
        assert_eq!(postal_address.city, "تهران");

        println!("{:?}", postal_address);
    }
}
