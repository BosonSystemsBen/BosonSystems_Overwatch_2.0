use crate::types::{ShipmentRequest, ShipmentResponse, ShipmentResponseData};

#[derive(Debug, thiserror::Error)]
pub enum SendcloudError {
    #[error("sendcloud request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("sendcloud API error ({status}): {message}")]
    Api { status: u16, message: String },
}

pub type SendcloudResult<T> = Result<T, SendcloudError>;

#[derive(serde::Deserialize)]
struct ErrorsBody {
    errors: Vec<crate::types::ErrorObject>,
}

fn extract_error_message(body: &str) -> String {
    serde_json::from_str::<ErrorsBody>(body)
        .ok()
        .and_then(|e| e.errors.into_iter().next())
        .and_then(|e| e.detail)
        .unwrap_or_else(|| body.to_string())
}

#[derive(Clone)]
pub struct SendcloudClient {
    http: reqwest::Client,
    base_url: String,
    public_key: String,
    private_key: String,
}

impl SendcloudClient {
    pub fn new(base_url: String, public_key: String, private_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            public_key,
            private_key,
        }
    }

    /// Announces a shipment (creates the label). If `request.external_reference_id`
    /// was already used before, Sendcloud returns the existing shipment (HTTP 409)
    /// instead of creating a new one — this call treats that as success.
    pub async fn announce_shipment(
        &self,
        request: &ShipmentRequest,
    ) -> SendcloudResult<ShipmentResponseData> {
        let url = format!("{}/shipments/announce", self.base_url);
        let response = self
            .http
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.private_key))
            .json(request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.as_u16() == 201 {
            let parsed: ShipmentResponse = serde_json::from_str(&body).map_err(|e| {
                SendcloudError::Api {
                    status: status.as_u16(),
                    message: format!("réponse inattendue de Sendcloud: {e}"),
                }
            })?;
            return Ok(parsed.data);
        }

        if status.as_u16() == 409 {
            let parsed: ShipmentResponseData = serde_json::from_str(&body).map_err(|e| {
                SendcloudError::Api {
                    status: status.as_u16(),
                    message: format!("réponse inattendue de Sendcloud: {e}"),
                }
            })?;
            return Ok(parsed);
        }

        Err(SendcloudError::Api {
            status: status.as_u16(),
            message: extract_error_message(&body),
        })
    }
}
