import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useMemo, useState } from "react"
import { api } from "../api"
import { errorMessage, useToast } from "../toast"
import { ShipmentDetail } from "./ShipmentDetail"
import { Button, Drawer, Input, Select, Table, Td, Th } from "./ui"

const STATUS_LABELS: Record<string, string> = {
  pending: "En attente",
  ready: "Prête à préparer",
  sent_to_sendcloud: "Envoyée à Sendcloud",
  shipped: "Expédiée",
}

export function ShipmentsPage() {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [statusFilter, setStatusFilter] = useState("")
  const [search, setSearch] = useState("")

  const shipments = useQuery({ queryKey: ["shipments"], queryFn: api.listShipments })

  const filtered = useMemo(() => {
    const term = search.trim().toLowerCase()
    return (shipments.data ?? []).filter((s) => {
      if (statusFilter && s.status !== statusFilter) return false
      if (!term) return true
      return s.invoice_number.toLowerCase().includes(term) || s.customer_name.toLowerCase().includes(term)
    })
  }, [shipments.data, statusFilter, search])

  const sync = useMutation({
    mutationFn: api.syncPennylane,
    onSuccess: (result) => {
      showToast("ok", `${result.invoices_processed} facture(s) traitée(s)`)
      queryClient.invalidateQueries({ queryKey: ["shipments"] })
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-end gap-3">
        <Button disabled={sync.isPending} onClick={() => sync.mutate()}>
          Synchroniser depuis Pennylane
        </Button>
        <Input
          placeholder="Rechercher (facture, client)..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="min-w-64"
        />
        <Select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
          <option value="">Tous les statuts</option>
          {Object.entries(STATUS_LABELS).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </Select>
      </div>

      <Table>
        <thead>
          <tr>
            <Th>Facture</Th>
            <Th>Client</Th>
            <Th>Statut</Th>
            <Th></Th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((s) => (
            <tr key={s.id}>
              <Td>{s.invoice_number}</Td>
              <Td>{s.customer_name}</Td>
              <Td>{STATUS_LABELS[s.status] ?? s.status}</Td>
              <Td>
                <Button variant="secondary" onClick={() => setSelectedId(s.id)}>
                  Détail
                </Button>
              </Td>
            </tr>
          ))}
          {filtered.length === 0 && (
            <tr>
              <Td colSpan={4} className="text-center text-slate-400">
                Aucune expédition ne correspond.
              </Td>
            </tr>
          )}
        </tbody>
      </Table>

      <Drawer open={!!selectedId} onClose={() => setSelectedId(null)} title="Détail de l'expédition">
        {selectedId && <ShipmentDetail id={selectedId} />}
      </Drawer>
    </div>
  )
}
