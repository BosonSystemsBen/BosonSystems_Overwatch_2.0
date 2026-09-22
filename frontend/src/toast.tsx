import { createContext, useCallback, useContext, useState, type ReactNode } from "react"
import { Banner } from "./components/ui"

interface Toast {
  id: number
  kind: "error" | "ok"
  message: string
}

const ToastContext = createContext<((kind: Toast["kind"], message: string) => void) | null>(null)

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([])

  const showToast = useCallback((kind: Toast["kind"], message: string) => {
    const id = Date.now() + Math.random()
    setToasts((prev) => [...prev, { id, kind, message }])
    setTimeout(() => setToasts((prev) => prev.filter((t) => t.id !== id)), 5000)
  }, [])

  return (
    <ToastContext.Provider value={showToast}>
      {children}
      <div className="fixed top-4 right-4 z-50 flex flex-col gap-2">
        {toasts.map((t) => (
          <Banner key={t.id} kind={t.kind}>
            {t.message}
          </Banner>
        ))}
      </div>
    </ToastContext.Provider>
  )
}

export function useToast() {
  const ctx = useContext(ToastContext)
  if (!ctx) throw new Error("useToast must be used within ToastProvider")
  return ctx
}

export function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err)
}
