import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { api } from "../api"
import { errorMessage, useToast } from "../toast"
import { Button, Card, Input, Label, Select, Table, Td, Th } from "./ui"

const STATUSES = ["pending", "ready", "sent_to_sendcloud", "shipped"]

export function ShipmentDetail({ id }: { id: string }) {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const [weightKg, setWeightKg] = useState("")
  const [selectedSerial, setSelectedSerial] = useState<Record<string, string>>({})

  const detail = useQuery({ queryKey: ["shipment", id], queryFn: () => api.getShipment(id) })
  const inStockSerials = useQuery({
    queryKey: ["serial-numbers", "in_stock"],
    queryFn: () => api.listSerialNumbers({ status: "in_stock" }),
  })

  const refresh = () => {
    queryClient.invalidateQueries({ queryKey: ["shipment", id] })
    queryClient.invalidateQueries({ queryKey: ["shipments"] })
    queryClient.invalidateQueries({ queryKey: ["serial-numbers"] })
  }

  const updateStatus = useMutation({
    mutationFn: (status: string) => api.updateShipmentStatus(id, status),
    onSuccess: () => {
      showToast("ok", "Statut mis à jour")
      refresh()
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  const linkSerial = useMutation({
    mutationFn: ({ lineId, serialId }: { lineId: string; serialId: string }) => api.linkSerialNumber(lineId, serialId),
    onSuccess: () => {
      showToast("ok", "S/N lié à la ligne")
      refresh()
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  const sendToSendcloud = useMutation({
    mutationFn: () => api.sendToSendcloud(id, Number(weightKg)),
    onSuccess: () => {
      showToast("ok", "Commande envoyée à Sendcloud")
      refresh()
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  if (!detail.data) return null
  const { shipment, lines, sendcloud_order } = detail.data

  return (
    <Card className="flex flex-col gap-4">
      <div>
        <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100">
          {shipment.invoice_number} — {shipment.customer_name}
        </h3>
        <p className="text-sm text-slate-500">
          {shipment.delivery_address}, {shipment.delivery_postal_code} {shipment.delivery_city} ({shipment.delivery_country})
        </p>
      </div>

      <Label>
        Statut
        <Select value={shipment.status} onChange={(e) => updateStatus.mutate(e.target.value)}>
          {STATUSES.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </Select>
      </Label>

      <Table>
        <thead>
          <tr>
            <Th>Article</Th>
            <Th>Qté</Th>
            <Th>Montant</Th>
            <Th>S/N lié</Th>
          </tr>
        </thead>
        <tbody>
          {lines.map((l) => {
            const options = inStockSerials.data?.filter((sn) => sn.product_id === l.product_id) ?? []
            return (
              <tr key={l.id}>
                <Td>
                  {l.label}
                  {!l.product_id && " (produit non reconnu)"}
                </Td>
                <Td>{l.quantity}</Td>
                <Td>{l.amount_eur ?? "—"} €</Td>
                <Td>
                  {options.length ? (
                    <div className="flex gap-2">
                      <Select
                        value={selectedSerial[l.id] ?? ""}
                        onChange={(e) => setSelectedSerial({ ...selectedSerial, [l.id]: e.target.value })}
                      >
                        <option value="">— choisir —</option>
                        {options.map((sn) => (
                          <option key={sn.id} value={sn.id}>
                            {sn.value}
                          </option>
                        ))}
                      </Select>
                      <Button
                        variant="secondary"
                        disabled={!selectedSerial[l.id]}
                        onClick={() => linkSerial.mutate({ lineId: l.id, serialId: selectedSerial[l.id] })}
                      >
                        Lier
                      </Button>
                    </div>
                  ) : (
                    "—"
                  )}
                </Td>
              </tr>
            )
          })}
        </tbody>
      </Table>

      <div>
        <h4 className="mb-2 text-sm font-semibold text-slate-900 dark:text-slate-100">Sendcloud</h4>
        {sendcloud_order ? (
          <p className="text-sm text-slate-500">
            Commande envoyée à Sendcloud (n° {sendcloud_order.order_number}, id {sendcloud_order.sendcloud_order_id}) —
            à compléter (douane si besoin) et étiqueter manuellement depuis le panel Sendcloud.
          </p>
        ) : (
          <form
            className="flex flex-col gap-3"
            onSubmit={(e) => {
              e.preventDefault()
              sendToSendcloud.mutate()
            }}
          >
            <p className="text-sm text-slate-500">
              Envoie la commande à Sendcloud pour validation humaine — aucune étiquette n'est créée automatiquement,
              aucun frais transporteur n'est engagé par cet appel.
            </p>
            <div className="flex items-end gap-3">
              <Label>
                Poids (kg)
                <Input
                  type="number"
                  step="0.001"
                  min="0.001"
                  required
                  value={weightKg}
                  onChange={(e) => setWeightKg(e.target.value)}
                />
              </Label>
              <Button type="submit" disabled={sendToSendcloud.isPending}>
                Envoyer à Sendcloud
              </Button>
            </div>
          </form>
        )}
      </div>
    </Card>
  )
}
