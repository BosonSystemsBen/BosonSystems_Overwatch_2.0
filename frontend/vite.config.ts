import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      '/products': 'http://127.0.0.1:8080',
      '/serial-numbers': 'http://127.0.0.1:8080',
      '/shipments': 'http://127.0.0.1:8080',
      '/shipment-lines': 'http://127.0.0.1:8080',
      '/pennylane': 'http://127.0.0.1:8080',
      '/health': 'http://127.0.0.1:8080',
    },
  },
})
