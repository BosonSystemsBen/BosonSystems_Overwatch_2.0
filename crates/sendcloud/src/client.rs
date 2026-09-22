use crate::types::{Order, OrderResponseItem, OrdersResponse};

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

    /// Pushes an order to Sendcloud for human review. This is an upsert on
    /// (order_id, integration.id): sending the same order again just updates
    /// it, it never creates a label or incurs a carrier charge by itself.
    pub async fn create_order(&self, order: &Order) -> SendcloudResult<OrderResponseItem> {
        let url = format!("{}/orders", self.base_url);
        let response = self
            .http
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.private_key))
            .json(&[order])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(SendcloudError::Api {
                status: status.as_u16(),
                message: extract_error_message(&body),
            });
        }

        let parsed: OrdersResponse = serde_json::from_str(&body).map_err(|e| SendcloudError::Api {
            status: status.as_u16(),
            message: format!("réponse inattendue de Sendcloud: {e}"),
        })?;

        parsed
            .data
            .into_iter()
            .next()
            .ok_or_else(|| SendcloudError::Api {
                status: status.as_u16(),
                message: "Sendcloud n'a renvoyé aucune commande".to_string(),
            })
    }
}
