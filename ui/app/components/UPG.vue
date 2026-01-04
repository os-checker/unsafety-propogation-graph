<template>
  <div :style="flowStyle">
    <Flow :raw="raw" />
  </div>
  <div :style="codeStyle">
    <CodeSrc :src="raw.src" />
    <CodeSrc :src="raw.mir" />
    <!-- <CodeMarkdown :doc="doc" /> -->
  </div>
</template>

<script setup lang="ts">
import { useWindowSize } from "@vueuse/core"
import type { Function } from "~/lib/output"
import { EMPTY_FUNCTION } from "~/lib/output"

const { width, height } = useWindowSize();
const flowWidthRatio = 0.9;
const flowHeightRatio = 0.6;
const flowStyle = computed(() => ({
  width: `${width.value * flowWidthRatio}px`,
  height: `${height.value * flowHeightRatio}px`,
}));
const codeStyle = computed(() => ({
  width: `${width.value * 0.98}px`,
  height: `${height.value * (1 - flowHeightRatio) * 0.9}px`,
  display: "grid",
  gridTemplateColumns: "1fr 1fr",
}));

const url = "https://raw.githubusercontent.com/os-checker/unsafety-propogation-graph-data/refs/heads/main/test/demo/function/S%3A%3Awrite_field.json"

const raw = ref<Function>(EMPTY_FUNCTION);
$fetch(url)
  .then(text => raw.value = JSON.parse(text as string))
  .catch(err => console.log(err));

watch(raw, val => console.log(val));

const doc = `
# Hello World

This is a *plain* **line**.

\`\`\`rust
fn main() {
    let a = String::new();
}
\`\`\`
`;
</script>
