<template>
  <div class="upg-left">
    <WidgetTop v-model:viewSelected="viewSelected" />
    <Flow :raw="raw" />
  </div>
  <div class="upg-right">
    <div class="upg-panel upg-panel-1">
      <WidgetSelectPanel v-model="leftPanel" :raw="raw" />
    </div>
    <div class="upg-panel">
      <WidgetSelectPanel v-model="rightPanel" :raw="raw" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Function } from "~/lib/output"
import { EMPTY_FUNCTION } from "~/lib/output"
import { Panel } from "~/lib/panel"
import { ALL_VIEW_TYPES, ViewType } from "~/lib/topbar";

const viewSelected = ref<ViewType[]>(ALL_VIEW_TYPES);

const url = "https://raw.githubusercontent.com/os-checker/unsafety-propagation-graph-data/refs/heads/main/test/poc/function/f.json"

const raw = ref<Function>(EMPTY_FUNCTION);
$fetch(url)
  .then(text => raw.value = JSON.parse(text as string))
  .catch(err => console.log(err));

const leftPanel = ref(Panel.Src);
const rightPanel = ref(Panel.Mir);
</script>
