import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { api } from "../api"
import { errorMessage, useToast } from "../toast"
import { ShipmentDetail } from "./ShipmentDetail"
import { Button, Table, Td, Th } from "./ui"

export function ShipmentsPage() {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const [selectedId, setSelectedId] = useState<string | null>(null)

  const shipments = useQuery({ queryKey: ["shipments"], queryFn: api.listShipments })

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
      <Button disabled={sync.isPending} onClick={() => sync.mutate()}>
        Synchroniser depuis Pennylane
      </Button>

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
          {shipments.data?.map((s) => (
            <tr key={s.id}>
              <Td>{s.invoice_number}</Td>
              <Td>{s.customer_name}</Td>
              <Td>{s.status}</Td>
              <Td>
                <Button variant="secondary" onClick={() => setSelectedId(s.id)}>
                  Détail
                </Button>
              </Td>
            </tr>
          ))}
        </tbody>
      </Table>

      {selectedId && <ShipmentDetail id={selectedId} />}
    </div>
  )
}
