import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useMemo, useState } from "react"
import { api, type Product } from "../api"
import { errorMessage, useToast } from "../toast"
import { Button, Card, Input, Label, Select, Table, Td, Th } from "./ui"

export function SerialNumbersPage() {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const [productId, setProductId] = useState("")
  const [value, setValue] = useState("")
  const [detectValue, setDetectValue] = useState("")
  const [detectResult, setDetectResult] = useState<Product[] | null>(null)
  const [search, setSearch] = useState("")

  const products = useQuery({ queryKey: ["products"], queryFn: api.listProducts })
  const serialNumbers = useQuery({ queryKey: ["serial-numbers"], queryFn: () => api.listSerialNumbers() })

  const productById = (id: string) => products.data?.find((p) => p.id === id)

  const filtered = useMemo(() => {
    const term = search.trim().toLowerCase()
    if (!term) return serialNumbers.data ?? []
    return (serialNumbers.data ?? []).filter((s) => {
      const product = productById(s.product_id)
      return (
        s.value.toLowerCase().includes(term) ||
        (s.assigned_to ?? "").toLowerCase().includes(term) ||
        (product?.sku ?? "").toLowerCase().includes(term) ||
        (product?.name ?? "").toLowerCase().includes(term)
      )
    })
  }, [serialNumbers.data, products.data, search])

  const createSerial = useMutation({
    mutationFn: () => api.createSerialNumber({ product_id: productId, value }),
    onSuccess: () => {
      showToast("ok", "S/N créé")
      setValue("")
      queryClient.invalidateQueries({ queryKey: ["serial-numbers"] })
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  const deleteSerial = useMutation({
    mutationFn: (id: string) => api.deleteSerialNumber(id),
    onSuccess: () => {
      showToast("ok", "S/N supprimé")
      queryClient.invalidateQueries({ queryKey: ["serial-numbers"] })
    },
    onError: (err) => showToast("error", errorMessage(err)),
  })

  const detect = useMutation({
    mutationFn: () => api.detectSerialNumber(detectValue),
    onSuccess: setDetectResult,
    onError: (err) => showToast("error", errorMessage(err)),
  })

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <form
          className="flex flex-wrap items-end gap-3"
          onSubmit={(e) => {
            e.preventDefault()
            detect.mutate()
          }}
        >
          <Label>
            Détecter un produit à partir d'un S/N
            <Input
              placeholder="PDU16-000123"
              value={detectValue}
              onChange={(e) => setDetectValue(e.target.value)}
            />
          </Label>
          <Button type="submit" variant="secondary" disabled={detect.isPending}>
            Détecter
          </Button>
          {detectResult && (
            <span className="text-sm text-slate-500">
              {detectResult.length ? `Détecté : ${detectResult.map((p) => p.sku).join(", ")}` : "Aucun produit ne correspond"}
            </span>
          )}
        </form>
      </Card>

      <Card>
        <form
          className="flex flex-wrap items-end gap-3"
          onSubmit={(e) => {
            e.preventDefault()
            createSerial.mutate()
          }}
        >
          <Label>
            Produit
            <Select required value={productId} onChange={(e) => setProductId(e.target.value)}>
              <option value="" disabled>
                — choisir —
              </option>
              {products.data?.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.sku} — {p.name}
                </option>
              ))}
            </Select>
          </Label>
          <Label>
            Valeur
            <Input required value={value} onChange={(e) => setValue(e.target.value)} />
          </Label>
          <Button type="submit" disabled={createSerial.isPending}>
            Créer
          </Button>
        </form>
      </Card>

      <Input
        placeholder="Rechercher (valeur, produit, client)..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        className="max-w-sm"
      />

      <Table>
        <thead>
          <tr>
            <Th>Valeur</Th>
            <Th>Produit</Th>
            <Th>Statut</Th>
            <Th>Assigné à</Th>
            <Th></Th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((s) => (
            <tr key={s.id}>
              <Td>{s.value}</Td>
              <Td>{productById(s.product_id)?.sku ?? s.product_id}</Td>
              <Td>{s.status}</Td>
              <Td>{s.assigned_to ?? "—"}</Td>
              <Td>
                <Button variant="danger" onClick={() => deleteSerial.mutate(s.id)}>
                  Supprimer
                </Button>
              </Td>
            </tr>
          ))}
          {filtered.length === 0 && (
            <tr>
              <Td colSpan={5} className="text-center text-slate-400">
                Aucun numéro de série ne correspond.
              </Td>
            </tr>
          )}
        </tbody>
      </Table>
    </div>
  )
}
