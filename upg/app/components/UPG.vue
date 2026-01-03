<template>
  <div :style="flowStyle">
    <Flow />
  </div>
  <div :style="codeStyle">
    <CodeSrc />
    <CodeMir />
  </div>
</template>

<script setup lang="ts">
import { useWindowSize } from '@vueuse/core'

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
  display: 'grid',
  gridTemplateColumns: '1fr 1fr',
}));
</script>
