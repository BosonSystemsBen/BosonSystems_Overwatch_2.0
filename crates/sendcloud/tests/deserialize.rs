use sendcloud::types::{
    Address, Integration, Measurement, Order, OrderDetails, OrderItem, OrderStatus,
    OrdersResponse, PaymentDetails, Price, ShippingDetails, Weight,
};

#[test]
fn deserializes_orders_response() {
    let json = r#"{
        "data": [
            { "id": 669, "order_id": "555413", "order_number": "OXSDFGHTD-12" }
        ]
    }"#;

    let parsed: OrdersResponse = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].id, 669);
    assert_eq!(parsed.data[0].order_id, "555413");
}

#[test]
fn serializes_order_request_matching_sendcloud_shape() {
    let order = Order {
        order_id: "shipment-uuid".to_string(),
        order_number: "F20260001".to_string(),
        order_details: OrderDetails {
            integration: Integration { id: 7 },
            status: OrderStatus {
                code: "to_review",
                message: "À valider avant expédition",
            },
            order_created_at: "2026-09-22T10:00:00+00:00".to_string(),
            order_items: vec![OrderItem {
                name: "PDU 16A".to_string(),
                quantity: 1,
                total_price: Price::eur(50.4),
            }],
        },
        payment_details: PaymentDetails {
            total_price: Price::eur(50.4),
            status: OrderStatus {
                code: "n/a",
                message: "Non suivi",
            },
        },
        shipping_address: Address {
            name: "Client ACME SARL".to_string(),
            address_line_1: "12 rue de la Paix".to_string(),
            postal_code: "75002".to_string(),
            city: "Paris".to_string(),
            country_code: "FR".to_string(),
            email: None,
            phone_number: None,
        },
        shipping_details: ShippingDetails {
            measurement: Measurement { weight: Weight::kg(1.2) },
        },
    };

    let json = serde_json::to_value(&order).unwrap();
    assert_eq!(json["order_id"], "shipment-uuid");
    assert_eq!(json["order_details"]["integration"]["id"], 7);
    assert_eq!(json["order_details"]["order_items"][0]["total_price"]["currency"], "EUR");
    assert_eq!(json["shipping_details"]["measurement"]["weight"]["value"], 1.2);
}
