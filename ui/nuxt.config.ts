// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  ssr: true,
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  sourcemap: { server: false, client: false, },
  icon: {
    serverBundle: { collections: ['tabler'] },
    clientBundle: {
      // https://icones.js.org/collection/codicon
      // https://icones.js.org/collection/tabler
      icons: [
        'tabler:letter-m', // module
        'tabler:letter-s', // struct
        'tabler:letter-e', // enum
        'tabler:letter-u', // union
        'tabler:letter-t', // trait
        'tabler:letter-t-small', // SelfTy
        'tabler:square-letter-f',// function
        'tabler:letter-m-small', // method
        'tabler:square-rounded-letter-p-filled', // safety property
        'tabler:alert-circle', // something wrong
      ],
    }
  },
  // devtools: { enabled: true }
})
