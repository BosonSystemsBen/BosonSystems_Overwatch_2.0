use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Address {
    pub name: String,
    pub address_line_1: String,
    pub postal_code: String,
    pub city: String,
    pub country_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Price {
    pub value: f64,
    pub currency: &'static str,
}

impl Price {
    pub fn eur(value: f64) -> Self {
        Self {
            value,
            currency: "EUR",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Integration {
    pub id: i64,
}

#[derive(Debug, Serialize)]
pub struct OrderStatus {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct OrderItem {
    pub name: String,
    pub quantity: i32,
    pub total_price: Price,
}

#[derive(Debug, Serialize)]
pub struct OrderDetails {
    pub integration: Integration,
    pub status: OrderStatus,
    pub order_created_at: String,
    pub order_items: Vec<OrderItem>,
}

#[derive(Debug, Serialize)]
pub struct PaymentDetails {
    pub total_price: Price,
    pub status: OrderStatus,
}

#[derive(Debug, Serialize)]
pub struct Weight {
    pub value: f64,
    pub unit: &'static str,
}

impl Weight {
    pub fn kg(value_kg: f64) -> Self {
        Self {
            value: value_kg,
            unit: "kg",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Measurement {
    pub weight: Weight,
}

#[derive(Debug, Serialize)]
pub struct ShippingDetails {
    pub measurement: Measurement,
}

/// An order pushed to Sendcloud for a human to review, complete customs
/// details on, and create the label from — this call never creates a label
/// or incurs any carrier charge by itself.
#[derive(Debug, Serialize)]
pub struct Order {
    pub order_id: String,
    pub order_number: String,
    pub order_details: OrderDetails,
    pub payment_details: PaymentDetails,
    pub shipping_address: Address,
    pub shipping_details: ShippingDetails,
}

#[derive(Debug, Deserialize)]
pub struct OrderResponseItem {
    pub id: i64,
    pub order_id: String,
    pub order_number: String,
}

#[derive(Debug, Deserialize)]
pub struct OrdersResponse {
    pub data: Vec<OrderResponseItem>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorObject {
    pub status: Option<String>,
    pub code: Option<String>,
    pub detail: Option<String>,
}
