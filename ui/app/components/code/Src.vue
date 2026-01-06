<template>
  <div v-html="renderedHtml"></div>
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
