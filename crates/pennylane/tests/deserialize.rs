use pennylane::types::{Customer, CustomerInvoice, CustomerInvoiceLine, Paginated};

#[test]
fn deserializes_customer_invoice_list_response() {
    let json = r#"{
        "items": [
            {
                "id": 42,
                "label": "Invoice label",
                "invoice_number": "F20230001",
                "currency": "EUR",
                "amount": "230.32",
                "date": "2023-08-30",
                "status": "paid",
                "draft": false,
                "customer": {
                    "id": 42,
                    "url": "https://app.pennylane.com/api/external/v2/customers/42"
                }
            }
        ],
        "has_more": false,
        "next_cursor": null
    }"#;

    let parsed: Paginated<CustomerInvoice> = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.items.len(), 1);
    assert_eq!(parsed.items[0].invoice_number, "F20230001");
    assert_eq!(parsed.items[0].customer.as_ref().unwrap().id, 42);
    assert!(!parsed.has_more);
}

#[test]
fn deserializes_invoice_line_with_product() {
    let json = r#"{
        "id": 444,
        "label": "Demo label",
        "unit": "piece",
        "quantity": "12",
        "amount": "50.4",
        "description": "Lorem ipsum",
        "product": {
            "id": 3049,
            "url": "https://app.pennylane.com/api/external/v2/products/42"
        },
        "vat_rate": "FR_200"
    }"#;

    let parsed: CustomerInvoiceLine = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.quantity, "12");
    assert_eq!(parsed.product.unwrap().id, 3049);
}

#[test]
fn deserializes_invoice_line_without_product() {
    let json = r#"{
        "id": 445,
        "label": "Discount",
        "unit": null,
        "quantity": "1",
        "amount": "-10.0",
        "description": "Remise",
        "product": null,
        "vat_rate": "FR_200"
    }"#;

    let parsed: CustomerInvoiceLine = serde_json::from_str(json).unwrap();
    assert!(parsed.product.is_none());
}

#[test]
fn deserializes_company_customer_with_delivery_address() {
    let json = r#"{
        "id": 42,
        "name": "My Company",
        "customer_type": "company",
        "delivery_address": {
            "address": "12 rue de la Paix",
            "postal_code": "75002",
            "city": "Paris",
            "country_alpha2": "FR"
        },
        "billing_address": {
            "address": "12 rue de la Paix",
            "postal_code": "75002",
            "city": "Paris",
            "country_alpha2": "FR"
        }
    }"#;

    let parsed: Customer = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.delivery_address.city, "Paris");
    assert_eq!(parsed.customer_type, "company");
}
