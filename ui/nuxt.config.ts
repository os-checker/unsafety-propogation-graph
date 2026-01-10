// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  ssr: true,
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  sourcemap: { server: false, client: false, },
  icon: {
    serverBundle: { collections: ['codicon'] },
    clientBundle: {
      // https://icones.js.org/collection/codicon
      icons: [
        'codicon:symbol-structure',
        'codicon:symbol-method',
        'codicon:symbol-class',
      ],
    }
  },
  // devtools: { enabled: true }
})
