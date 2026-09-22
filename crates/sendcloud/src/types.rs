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
pub struct FromAddress {
    pub sender_address_id: i64,
}

#[derive(Debug, Serialize)]
pub struct ShippingOptionCodeProperties {
    pub shipping_option_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ShipWith {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub properties: ShippingOptionCodeProperties,
}

impl ShipWith {
    pub fn shipping_option_code(shipping_option_code: String, contract_id: Option<i64>) -> Self {
        Self {
            kind: "shipping_option_code",
            properties: ShippingOptionCodeProperties {
                shipping_option_code,
                contract_id,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Weight {
    pub value: String,
    pub unit: &'static str,
}

impl Weight {
    pub fn kg(value_kg: f64) -> Self {
        Self {
            value: format!("{value_kg:.3}"),
            unit: "kg",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Parcel {
    pub weight: Weight,
}

#[derive(Debug, Serialize)]
pub struct ShipmentRequest {
    pub to_address: Address,
    pub from_address: FromAddress,
    pub ship_with: ShipWith,
    pub parcels: Vec<Parcel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_reference_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Document {
    pub document_type: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ParcelResponse {
    pub id: i64,
    pub status: Option<Status>,
    #[serde(default)]
    pub documents: Vec<Document>,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    /// Base64-encoded label file, only present when the shipment has a single parcel.
    pub label_file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Carrier {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorObject {
    pub status: Option<String>,
    pub code: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShipmentResponseData {
    pub id: String,
    #[serde(default)]
    pub parcels: Vec<ParcelResponse>,
    pub carrier: Option<Carrier>,
    #[serde(default)]
    pub errors: Vec<ErrorObject>,
}

#[derive(Debug, Deserialize)]
pub struct ShipmentResponse {
    pub data: ShipmentResponseData,
}
