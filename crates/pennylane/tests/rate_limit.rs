use pennylane::PennylaneClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn retries_after_429_and_succeeds() {
    let server = MockServer::start().await;

    // First request: rate limited.
    Mock::given(method("GET"))
        .and(path("/api/external/v2/customers/42"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "0")
                .set_body_string("Rate limit exceeded. Please retry in 1 second."),
        )
        .up_to_n_times(1)
        .with_priority(1)
        .expect(1)
        .mount(&server)
        .await;

    // Second request: succeeds.
    Mock::given(method("GET"))
        .and(path("/api/external/v2/customers/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Client ACME SARL",
            "customer_type": "company",
            "delivery_address": { "address": "12 rue de la Paix", "postal_code": "75002", "city": "Paris", "country_alpha2": "FR" },
            "billing_address": { "address": "12 rue de la Paix", "postal_code": "75002", "city": "Paris", "country_alpha2": "FR" }
        })))
        .with_priority(2)
        .expect(1)
        .mount(&server)
        .await;

    let client = PennylaneClient::new(server.uri(), "test-token".to_string());
    let customer = client.get_customer(42).await.expect("should retry then succeed");

    assert_eq!(customer.name, "Client ACME SARL");
}
