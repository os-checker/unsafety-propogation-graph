<template>
  <div v-html="renderedHtml" class="upg-code-src"></div>
</template>

<script setup lang="ts">
const { $shiki } = useNuxtApp()
const { src } = defineProps<{ src: string }>();

// Don't move hlOpts to computed closure, because it'll re-hightlight code.
const hlOpts = {
  lang: "rust",
  themes: globalTheme().shikiThemes
};
const renderedHtml = computed(() => {
  return $shiki.highlighter.codeToHtml(src, hlOpts);
})
</script>

<style lang="css">
.upg-code-src pre,
.upg-code-src code {
  white-space: pre-wrap !important;
  word-break: break-all !important;
  overflow-wrap: break-word !important;
  max-width: 100%;
}
</style>
