<template>
  <div class="upg-panel-header">
    <USelect v-model="selected" :items="PANELS" placeholder="Select Panel" class="w-40"
      :content="{ bodyLock: false }" />
  </div>

  <div class="upg-panel-content">
    <CodeSrc v-if="selected === Panel.Src" :src="raw.src" />
    <CodeSrc v-else-if="selected === Panel.Mir" :src="raw.mir" />
    <CodeMarkdown v-else-if="selected === Panel.Doc" :doc="raw.doc" />
  </div>
</template>

<script setup lang="ts">
import { Panel, PANELS } from "~/lib/panel"
import type { Function } from "~/lib/output"

const selected = defineModel<Panel>();

const props = defineProps<{ raw: Function }>();
</script>
