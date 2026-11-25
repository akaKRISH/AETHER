import { defineConfig, loadEnv } from 'vite'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', 'AETHER_WEB_')
  const proxy = {
    '/v1': env.AETHER_WEB_API_ORIGIN || 'http://127.0.0.1:8080',
  }
  return { server: { proxy }, preview: { proxy } }
})
