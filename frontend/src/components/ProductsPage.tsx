import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { api } from "../api"
import { errorMessage, useToast } from "../toast"
import { Button, Card, Code, Input, Label, Table, Td, Th } from "./ui"

const emptyForm = { sku: "", name: "", requires_serial: false, detection_pattern: "", pennylane_product_id: "" }

export function ProductsPage() {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const [form, setForm] = useState(emptyForm)

  const products = useQuery({ queryKey: ["products"], queryFn: api.listProducts })

  const createProduct = useMutation({
    mutationFn: () =>
      api.createProduct({
        sku: form.sku,
        name: form.name,
        requires_serial: form.requires_serial,
        detection_pattern: form.detection_pattern || null,
        pennylane_product_id: form.pennylane_product_id ? Number(form.pennylane_product_id) : null,
      }),
    onSuccess: () => {
      showToast("ok", "Produit créé")
      setForm(emptyForm)
      queryClient.invalidateQueries({ queryKey: ["products"] })
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  const deleteProduct = useMutation({
    mutationFn: (id: string) => api.deleteProduct(id),
    onSuccess: () => {
      showToast("ok", "Produit supprimé")
      queryClient.invalidateQueries({ queryKey: ["products"] })
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <form
          className="flex flex-wrap items-end gap-3"
          onSubmit={(e) => {
            e.preventDefault()
            createProduct.mutate()
          }}
        >
          <Label>
            SKU
            <Input required value={form.sku} onChange={(e) => setForm({ ...form, sku: e.target.value })} />
          </Label>
          <Label>
            Nom
            <Input required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
          </Label>
          <Label className="flex-row items-center gap-2">
            <input
              type="checkbox"
              checked={form.requires_serial}
              onChange={(e) => setForm({ ...form, requires_serial: e.target.checked })}
            />
            Requiert S/N
          </Label>
          <Label>
            Motif de détection (regex)
            <Input
              placeholder="^PDU16-[0-9]{6}$"
              value={form.detection_pattern}
              onChange={(e) => setForm({ ...form, detection_pattern: e.target.value })}
            />
          </Label>
          <Label>
            ID produit Pennylane
            <Input
              type="number"
              value={form.pennylane_product_id}
              onChange={(e) => setForm({ ...form, pennylane_product_id: e.target.value })}
            />
          </Label>
          <Button type="submit" disabled={createProduct.isPending}>
            Créer
          </Button>
        </form>
      </Card>

      <Table>
        <thead>
          <tr>
            <Th>SKU</Th>
            <Th>Nom</Th>
            <Th>S/N requis</Th>
            <Th>Motif</Th>
            <Th>ID Pennylane</Th>
            <Th></Th>
          </tr>
        </thead>
        <tbody>
          {products.data?.map((p) => (
            <tr key={p.id}>
              <Td>{p.sku}</Td>
              <Td>{p.name}</Td>
              <Td>{p.requires_serial ? "oui" : "non"}</Td>
              <Td>{p.detection_pattern ? <Code>{p.detection_pattern}</Code> : "—"}</Td>
              <Td>{p.pennylane_product_id ?? "—"}</Td>
              <Td>
                <Button variant="danger" onClick={() => deleteProduct.mutate(p.id)}>
                  Supprimer
                </Button>
              </Td>
            </tr>
          ))}
        </tbody>
      </Table>
    </div>
  )
}
