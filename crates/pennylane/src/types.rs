use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdRef {
    pub id: i64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct CustomerInvoice {
    pub id: i64,
    pub invoice_number: String,
    pub date: Option<String>,
    pub status: String,
    pub draft: bool,
    pub customer: Option<IdRef>,
}

#[derive(Debug, Deserialize)]
pub struct CustomerInvoiceLine {
    pub id: i64,
    pub label: String,
    pub description: String,
    /// Pennylane returns this as a string (e.g. "12"), not a number.
    pub quantity: String,
    /// Total line amount in euros, as a string (e.g. "50.4").
    pub amount: String,
    pub product: Option<IdRef>,
}

#[derive(Debug, Deserialize)]
pub struct Address {
    pub address: String,
    pub postal_code: String,
    pub city: String,
    pub country_alpha2: String,
}

#[derive(Debug, Deserialize)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    pub customer_type: String,
    pub delivery_address: Address,
    pub billing_address: Address,
}
