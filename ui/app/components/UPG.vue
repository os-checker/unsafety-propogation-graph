<template>
  <div class="upg-left">
    <Flow :raw="raw" />
  </div>
  <div class="upg-right">
    <div class="relative">
      <WidgetSelectPanel v-model="leftPanel" :raw="raw" />
    </div>
    <div class="relative">
      <WidgetSelectPanel v-model="rightPanel" :raw="raw" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useWindowSize } from "@vueuse/core"
import type { Function } from "~/lib/output"
import { EMPTY_FUNCTION } from "~/lib/output"
import { Panel } from "~/lib/panel"

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

const url = "https://raw.githubusercontent.com/os-checker/unsafety-propagation-graph-data/refs/heads/main/test/poc/function/f.json"

const raw = ref<Function>(EMPTY_FUNCTION);
$fetch(url)
  .then(text => raw.value = JSON.parse(text as string))
  .catch(err => console.log(err));

const leftPanel = ref(Panel.Src);
const rightPanel = ref(Panel.Mir);
</script>
