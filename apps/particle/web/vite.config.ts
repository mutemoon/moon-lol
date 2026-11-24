import fs from 'fs'
import path from 'path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, type Plugin } from 'vite'

const mimeMap: Record<string, string> = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.dds': 'image/vnd-ms.dds',
  '.tga': 'image/x-tga',
  '.spv': 'application/x-spirv',
  '.scb': 'application/octet-stream',
  '.sco': 'application/octet-stream',
  '.ron': 'text/plain',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
}

function serveWorkspaceAssetsPlugin(): Plugin {
  return {
    name: 'serve-workspace-assets',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url && (req.url.startsWith('/assets/') || req.url.startsWith('/wasm/assets/'))) {
          const rawPath = req.url.split('?')[0].replace(/^(\/wasm)?\/assets\//, '')
          const decoded = decodeURIComponent(rawPath)
          const workspaceAssetsDir = path.resolve(import.meta.dirname, '../../../assets')
          const fullPath = path.join(workspaceAssetsDir, decoded)

          if (fs.existsSync(fullPath) && fs.statSync(fullPath).isFile()) {
            const ext = path.extname(fullPath).toLowerCase()
            const contentType = mimeMap[ext] || 'application/octet-stream'
            res.setHeader('Content-Type', contentType)
            res.setHeader('Access-Control-Allow-Origin', '*')
            fs.createReadStream(fullPath).pipe(res)
            return
          } else {
            // 文件不存在时直接返回 404，绝不 fallback 到 SPA 的 index.html
            res.statusCode = 404
            res.end(`Asset not found: ${decoded}`)
            return
          }
        }
        next()
      })
    },
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), serveWorkspaceAssetsPlugin()],
  resolve: {
    alias: {
      '@': path.resolve(import.meta.dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/wasm': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/wasm/, ''),
      },
      '^/particle-.*': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/favicon.ico': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
})
