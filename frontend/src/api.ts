export interface Product {
  id: string
  sku: string
  name: string
  requires_serial: boolean
  detection_pattern: string | null
  pennylane_product_id: number | null
  created_at: string
  updated_at: string
}

export interface SerialNumber {
  id: string
  product_id: string
  value: string
  status: string
  assigned_to: string | null
  shipment_line_id: string | null
  created_at: string
  updated_at: string
}

export interface Shipment {
  id: string
  pennylane_invoice_id: number
  invoice_number: string
  pennylane_customer_id: number
  customer_name: string
  delivery_address: string
  delivery_postal_code: string
  delivery_city: string
  delivery_country: string
  status: string
  created_at: string
  updated_at: string
}

export interface ShipmentLine {
  id: string
  shipment_id: string
  pennylane_line_id: number
  product_id: string | null
  label: string
  quantity: string
  amount_eur: string | null
  created_at: string
  updated_at: string
}

export interface SendcloudOrder {
  id: string
  shipment_id: string
  sendcloud_order_id: number
  order_number: string
  created_at: string
  updated_at: string
}

export interface ShipmentDetail {
  shipment: Shipment
  lines: ShipmentLine[]
  sendcloud_order: SendcloudOrder | null
}

async function apiFetch<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(path, {
    headers: { "Content-Type": "application/json" },
    ...options,
  })
  const text = await res.text()
  const data = text ? JSON.parse(text) : null
  if (!res.ok) {
    throw new Error(data?.error ?? `HTTP ${res.status}`)
  }
  return data as T
}

export const api = {
  listProducts: () => apiFetch<Product[]>("/products"),
  createProduct: (input: {
    sku: string
    name: string
    requires_serial: boolean
    detection_pattern: string | null
    pennylane_product_id: number | null
  }) => apiFetch<Product>("/products", { method: "POST", body: JSON.stringify(input) }),
  updateProduct: (
    id: string,
    input: Partial<{
      sku: string
      name: string
      requires_serial: boolean
      detection_pattern: string | null
      pennylane_product_id: number | null
    }>,
  ) => apiFetch<Product>(`/products/${id}`, { method: "PUT", body: JSON.stringify(input) }),
  deleteProduct: (id: string) => apiFetch<null>(`/products/${id}`, { method: "DELETE" }),

  listSerialNumbers: (params?: { product_id?: string; status?: string }) => {
    const query = new URLSearchParams(params as Record<string, string>).toString()
    return apiFetch<SerialNumber[]>(`/serial-numbers${query ? `?${query}` : ""}`)
  },
  createSerialNumber: (input: { product_id: string; value: string }) =>
    apiFetch<SerialNumber>("/serial-numbers", { method: "POST", body: JSON.stringify(input) }),
  deleteSerialNumber: (id: string) => apiFetch<null>(`/serial-numbers/${id}`, { method: "DELETE" }),
  detectSerialNumber: (value: string) =>
    apiFetch<Product[]>("/serial-numbers/detect", { method: "POST", body: JSON.stringify({ value }) }),

  listShipments: () => apiFetch<Shipment[]>("/shipments"),
  getShipment: (id: string) => apiFetch<ShipmentDetail>(`/shipments/${id}`),
  updateShipmentStatus: (id: string, status: string) =>
    apiFetch<Shipment>(`/shipments/${id}/status`, { method: "PUT", body: JSON.stringify({ status }) }),
  linkSerialNumber: (lineId: string, serialNumberId: string) =>
    apiFetch<SerialNumber>(`/shipment-lines/${lineId}/link-serial-number`, {
      method: "POST",
      body: JSON.stringify({ serial_number_id: serialNumberId }),
    }),
  syncPennylane: () => apiFetch<{ invoices_processed: number }>("/pennylane/sync", { method: "POST" }),
  sendToSendcloud: (id: string, weightKg: number) =>
    apiFetch<SendcloudOrder>(`/shipments/${id}/send-to-sendcloud`, {
      method: "POST",
      body: JSON.stringify({ weight_kg: weightKg }),
    }),
}
