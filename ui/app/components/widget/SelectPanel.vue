<template>
  <div class="upg-panel-header">
    <USelect v-model="selected" :items="PANELS" placeholder="Select Panel" class="w-40"
      :content="{ bodyLock: false }" />

    <UCheckbox label="Wrap" v-model="isWrapped" />
  </div>

  <div class="upg-panel-content">
    <CodeSrc v-if="selected === Panel.Src" :src="raw.src" :isWrapped="isWrapped" />
    <CodeSrc v-else-if="selected === Panel.Mir" :src="raw.mir" :isWrapped="isWrapped" />
    <CodeSrc v-else-if="selected === Panel.Raw" :src="JSON.stringify(raw, undefined, 2)" :isWrapped="isWrapped" />
    <CodeMarkdown v-else-if="selected === Panel.Doc" :doc="raw.doc" />
  </div>
</template>

<script setup lang="ts">
import { Panel, PANELS } from "~/lib/panel"
import type { Function } from "~/lib/output"

const selected = defineModel<Panel>();

const props = defineProps<{ raw: Function }>();

const isWrapped = ref(true);
</script>
