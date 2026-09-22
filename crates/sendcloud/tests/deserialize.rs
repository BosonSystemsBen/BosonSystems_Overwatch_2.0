use sendcloud::types::ShipmentResponse;

#[test]
fn deserializes_successful_shipment_response() {
    let json = r#"{
        "data": {
            "id": "XXX-Shipment-id",
            "parcels": [
                {
                    "id": 383707309,
                    "status": { "code": "READY_TO_SEND", "message": "Ready to send" },
                    "documents": [
                        { "document_type": "label", "link": "https://panel.sendcloud.sc/api/v3/parcels/383707309/documents/label" }
                    ],
                    "tracking_number": "3SYZXG8498635",
                    "tracking_url": "https://tracking.eu-central-1-0.sendcloud.sc/forward?carrier=postnl",
                    "label_file": "JVBERi0xLjQK..."
                }
            ],
            "carrier": { "code": "postnl", "name": "PostNL" },
            "errors": []
        }
    }"#;

    let parsed: ShipmentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.data.id, "XXX-Shipment-id");
    assert_eq!(parsed.data.parcels.len(), 1);
    assert_eq!(parsed.data.parcels[0].tracking_number.as_deref(), Some("3SYZXG8498635"));
    assert!(parsed.data.parcels[0].label_file.is_some());
    assert!(parsed.data.errors.is_empty());
}

#[test]
fn deserializes_failed_announcement_response() {
    let json = r#"{
        "data": {
            "id": "XXX-Shipment-id",
            "parcels": [
                {
                    "id": 383707309,
                    "status": { "code": "ANNOUNCEMENT_FAILED", "message": "Announcement Failed" },
                    "tracking_number": "3SYZXG8498635"
                }
            ],
            "carrier": { "code": "postnl", "name": "PostNL" },
            "errors": [
                { "status": "500", "code": "parcel_announcement_error", "detail": "Service error: An error occurred while connecting to the carrier." }
            ]
        }
    }"#;

    let parsed: ShipmentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.data.parcels[0].status.as_ref().unwrap().code, "ANNOUNCEMENT_FAILED");
    assert_eq!(parsed.data.errors.len(), 1);
}
