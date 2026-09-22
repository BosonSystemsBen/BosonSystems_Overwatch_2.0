import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query"
import { useMemo, useRef, useState } from "react"
import { api, type ShipmentLine } from "../api"
import { errorMessage } from "../toast"
import { Button, Card, Input, Label, Table, Td, Th } from "./ui"

type ScanLogEntry = {
  id: string
  value: string
  ok: boolean
  message: string
}

function newLogId() {
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`
}

export function PackingPage() {
  const queryClient = useQueryClient()
  const [shipmentId, setShipmentId] = useState<string | null>(null)
  const [scanValue, setScanValue] = useState("")
  const [log, setLog] = useState<ScanLogEntry[]>([])
  const inputRef = useRef<HTMLInputElement>(null)

  const shipments = useQuery({ queryKey: ["shipments"], queryFn: api.listShipments })
  const products = useQuery({ queryKey: ["products"], queryFn: api.listProducts })
  const readyShipments = useMemo(() => (shipments.data ?? []).filter((s) => s.status === "ready"), [shipments.data])

  const detail = useQuery({
    queryKey: ["shipment", shipmentId],
    queryFn: () => api.getShipment(shipmentId as string),
    enabled: !!shipmentId,
  })

  const lines = detail.data?.lines ?? []
  const productById = (id: string | null) => (id ? products.data?.find((p) => p.id === id) : undefined)

  const lineLinkedQueries = useQueries({
    queries: lines.map((l) => ({
      queryKey: ["serial-numbers", "line", l.id],
      queryFn: () => api.listSerialNumbers({ shipment_line_id: l.id }),
      enabled: !!shipmentId,
    })),
  })

  const refresh = () => {
    queryClient.invalidateQueries({ queryKey: ["shipment", shipmentId] })
    queryClient.invalidateQueries({ queryKey: ["shipments"] })
    queryClient.invalidateQueries({ queryKey: ["serial-numbers"] })
  }

  const pushLog = (entry: Omit<ScanLogEntry, "id">) => {
    setLog((prev) => [{ id: newLogId(), ...entry }, ...prev].slice(0, 25))
  }

  const scan = useMutation({
    mutationFn: async (rawValue: string) => {
      const value = rawValue.trim()
      if (!value) throw new Error("Valeur vide")
      if (!detail.data) throw new Error("Sélectionnez une expédition à préparer")

      const existing = await api.listSerialNumbers({ value })
      let serial = existing[0]

      if (serial) {
        if (serial.shipment_line_id) {
          throw new Error(`Ce S/N est déjà lié ailleurs (statut « ${serial.status} »).`)
        }
      } else {
        const matches = await api.detectSerialNumber(value)
        if (matches.length === 0) {
          throw new Error("Aucun produit ne correspond à ce S/N — vérifiez la nomenclature.")
        }
        if (matches.length > 1) {
          throw new Error(`Motif ambigu (${matches.map((m) => m.sku).join(", ")}) — créez le S/N manuellement.`)
        }
        serial = await api.createSerialNumber({ product_id: matches[0].id, value })
      }

      const eligibleLines = detail.data.lines.filter((l) => l.product_id === serial!.product_id)
      if (eligibleLines.length === 0) {
        throw new Error("Aucune ligne de cette expédition n'attend ce produit.")
      }

      let targetLine: ShipmentLine | undefined
      for (const line of eligibleLines) {
        const linked = await api.listSerialNumbers({ shipment_line_id: line.id })
        const needed = Math.round(Number(line.quantity)) || 1
        if (linked.length < needed) {
          targetLine = line
          break
        }
      }
      if (!targetLine) {
        throw new Error("Cette ligne est déjà complète pour cette expédition.")
      }

      await api.linkSerialNumber(targetLine.id, serial.id)
      return { serial, line: targetLine }
    },
    onSuccess: ({ serial, line }) => {
      pushLog({ value: serial.value, ok: true, message: `Lié à « ${line.label} »` })
      setScanValue("")
      refresh()
      inputRef.current?.focus()
    },
    onError: (err, value) => {
      pushLog({ value, ok: false, message: errorMessage(err) })
      setScanValue("")
      inputRef.current?.focus()
    },
  })

  return (
    <div className="flex flex-col gap-4 lg:flex-row">
      <Card className="flex-shrink-0 lg:w-72">
        <h3 className="mb-3 text-sm font-semibold text-slate-900 dark:text-slate-100">Prêtes à préparer</h3>
        <ul className="flex flex-col gap-1">
          {readyShipments.map((s) => (
            <li key={s.id}>
              <button
                type="button"
                onClick={() => setShipmentId(s.id)}
                className={`w-full rounded-md px-2.5 py-2 text-left text-sm transition-colors ${
                  shipmentId === s.id
                    ? "bg-slate-900 text-white dark:bg-slate-100 dark:text-slate-900"
                    : "hover:bg-slate-100 dark:hover:bg-slate-800"
                }`}
              >
                <div className="font-medium">{s.invoice_number}</div>
                <div className={shipmentId === s.id ? "text-slate-300" : "text-slate-500"}>{s.customer_name}</div>
              </button>
            </li>
          ))}
          {readyShipments.length === 0 && (
            <li className="px-2.5 py-2 text-sm text-slate-400">Aucune expédition prête pour l'instant.</li>
          )}
        </ul>
      </Card>

      <div className="flex flex-1 flex-col gap-4">
        {!shipmentId && (
          <Card>
            <p className="text-sm text-slate-500">Sélectionnez une expédition à préparer dans la liste.</p>
          </Card>
        )}

        {shipmentId && detail.data && (
          <>
            <Card>
              <h3 className="mb-1 text-base font-semibold text-slate-900 dark:text-slate-100">
                {detail.data.shipment.invoice_number} — {detail.data.shipment.customer_name}
              </h3>
              <p className="mb-3 text-sm text-slate-500">
                Scannez (ou tapez) un numéro de série : détection, création si besoin, et liaison à la bonne ligne se
                font en une seule fois.
              </p>
              <form
                className="flex items-end gap-3"
                onSubmit={(e) => {
                  e.preventDefault()
                  if (scanValue.trim() && !scan.isPending) scan.mutate(scanValue)
                }}
              >
                <Label className="flex-1">
                  Numéro de série
                  <Input
                    ref={inputRef}
                    autoFocus
                    value={scanValue}
                    onChange={(e) => setScanValue(e.target.value)}
                    placeholder="PDU16-000123"
                  />
                </Label>
                <Button type="submit" disabled={scan.isPending}>
                  Valider
                </Button>
              </form>
            </Card>

            <Table>
              <thead>
                <tr>
                  <Th>Article</Th>
                  <Th>Besoin</Th>
                  <Th>Statut</Th>
                </tr>
              </thead>
              <tbody>
                {lines.map((l, i) => {
                  const product = productById(l.product_id)
                  const needed = Math.round(Number(l.quantity)) || 1
                  const linked = lineLinkedQueries[i]?.data
                  return (
                    <tr key={l.id}>
                      <Td>
                        {l.label}
                        {!l.product_id && " (produit non reconnu)"}
                      </Td>
                      <Td>{needed}</Td>
                      <Td>
                        {!product?.requires_serial ? (
                          "— (pas de S/N requis)"
                        ) : linked === undefined ? (
                          "…"
                        ) : linked.length >= needed ? (
                          <span className="text-emerald-600 dark:text-emerald-400">
                            Complet ({linked.length}/{needed})
                          </span>
                        ) : (
                          <span className="text-amber-600 dark:text-amber-400">
                            {linked.length}/{needed}
                          </span>
                        )}
                      </Td>
                    </tr>
                  )
                })}
              </tbody>
            </Table>

            <Card>
              <h4 className="mb-2 text-sm font-semibold text-slate-900 dark:text-slate-100">Historique de scan</h4>
              {log.length === 0 ? (
                <p className="text-sm text-slate-400">Aucun scan pour l'instant.</p>
              ) : (
                <ul className="flex flex-col gap-1 text-sm">
                  {log.map((entry) => (
                    <li key={entry.id} className={entry.ok ? "text-emerald-600 dark:text-emerald-400" : "text-red-600 dark:text-red-400"}>
                      <span className="font-mono">{entry.value}</span> — {entry.message}
                    </li>
                  ))}
                </ul>
              )}
            </Card>
          </>
        )}
      </div>
    </div>
  )
}
