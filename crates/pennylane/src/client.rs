use crate::types::{CustomerInvoice, CustomerInvoiceLine, Customer, Paginated};

#[derive(Debug, thiserror::Error)]
pub enum PennylaneError {
    #[error("pennylane request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("pennylane API error ({status}): {message}")]
    Api { status: u16, message: String },
}

pub type PennylaneResult<T> = Result<T, PennylaneError>;

/// Pennylane allows 5 req/s on the Company API V2. On a 429 we back off and
/// retry rather than aborting the whole sync.
const MAX_RATE_LIMIT_RETRIES: u32 = 5;

#[derive(Clone)]
pub struct PennylaneClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl PennylaneClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            token,
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> PennylaneResult<T> {
        let url = format!("{}{}", self.base_url, path);

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            let response = self
                .http
                .get(&url)
                .bearer_auth(&self.token)
                .query(query)
                .send()
                .await?;

            let status = response.status();

            if status.as_u16() == 429 && attempt < MAX_RATE_LIMIT_RETRIES {
                let retry_after_secs = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(1);
                tokio::time::sleep(std::time::Duration::from_millis(retry_after_secs * 1000 + 200))
                    .await;
                continue;
            }

            if !status.is_success() {
                let message = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<no body>".to_string());
                return Err(PennylaneError::Api {
                    status: status.as_u16(),
                    message,
                });
            }

            return Ok(response.json::<T>().await?);
        }

        unreachable!("the loop above always returns before exhausting its retries")
    }

    /// Lists finalized (non-draft) customer invoices, most recently created first —
    /// recent orders matter more than working through years of old history.
    pub async fn list_customer_invoices(
        &self,
        cursor: Option<&str>,
    ) -> PennylaneResult<Paginated<CustomerInvoice>> {
        let mut query = vec![
            ("sort".to_string(), "-id".to_string()),
            (
                "filter".to_string(),
                r#"[{"field":"draft","operator":"eq","value":"false"}]"#.to_string(),
            ),
        ];
        if let Some(cursor) = cursor {
            query.push(("cursor".to_string(), cursor.to_string()));
        }
        self.get_json("/api/external/v2/customer_invoices", &query)
            .await
    }

    pub async fn list_invoice_lines(
        &self,
        invoice_id: i64,
        cursor: Option<&str>,
    ) -> PennylaneResult<Paginated<CustomerInvoiceLine>> {
        let mut query = vec![("sort".to_string(), "rank".to_string())];
        if let Some(cursor) = cursor {
            query.push(("cursor".to_string(), cursor.to_string()));
        }
        self.get_json(
            &format!("/api/external/v2/customer_invoices/{invoice_id}/invoice_lines"),
            &query,
        )
        .await
    }

    pub async fn get_customer(&self, customer_id: i64) -> PennylaneResult<Customer> {
        self.get_json::<Customer>(&format!("/api/external/v2/customers/{customer_id}"), &[])
            .await
    }
}
