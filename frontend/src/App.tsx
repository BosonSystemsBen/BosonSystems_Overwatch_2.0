import { useState } from "react"
import { PackingPage } from "./components/PackingPage"
import { ProductsPage } from "./components/ProductsPage"
import { SerialNumbersPage } from "./components/SerialNumbersPage"
import { ShipmentsPage } from "./components/ShipmentsPage"
import { ToastProvider } from "./toast"

const TABS = [
  { id: "packing", label: "Préparation", render: () => <PackingPage /> },
  { id: "shipments", label: "Expéditions", render: () => <ShipmentsPage /> },
  { id: "products", label: "Produits", render: () => <ProductsPage /> },
  { id: "serials", label: "Numéros de série", render: () => <SerialNumbersPage /> },
] as const

export default function App() {
  const [tab, setTab] = useState<(typeof TABS)[number]["id"]>("packing")

  return (
    <ToastProvider>
      <div className="min-h-svh bg-slate-50 text-slate-900 dark:bg-slate-950 dark:text-slate-100">
        <header className="border-b border-slate-200 bg-slate-900 px-6 py-4 dark:border-slate-800">
          <h1 className="text-lg font-semibold text-white">Overwatch</h1>
        </header>
        <nav className="flex gap-1 border-b border-slate-200 bg-white px-4 dark:border-slate-800 dark:bg-slate-900">
          {TABS.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={`px-4 py-3 text-sm font-medium transition-colors ${
                tab === t.id
                  ? "border-b-2 border-slate-900 text-slate-900 dark:border-white dark:text-white"
                  : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
              }`}
            >
              {t.label}
            </button>
          ))}
        </nav>
        <main className="mx-auto max-w-5xl p-6">{TABS.find((t) => t.id === tab)?.render()}</main>
      </div>
    </ToastProvider>
  )
}
