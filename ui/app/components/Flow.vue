<template>
  <VueFlow :nodes="data.nodes" :edges="data.edges" @nodes-initialized="layoutGraph('LR')"></VueFlow>
</template>

<script setup lang="ts">
import type { Node, Edge } from '@vue-flow/core'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import type { Function } from "~/lib/output"

const props = defineProps<{ raw: Function }>();

const { fitView } = useVueFlow();
const { layout } = useLayout();

type Data = { nodes: Node[], edges: Edge[] };
const EMPTY_DATA = { nodes: [], edges: [] };

const data = ref<Data>(EMPTY_DATA);

watch(() => props.raw, val => {
  if (!val.name) return;

  // Placeholder for initial position. The layout will be recomputed later.
  const POS = { x: 0, y: 0 };

  // Add the current function as root node, callees and adts as leaves.
  const root: Node = { id: val.name, label: val.name, position: POS };
  const callees: Node[] = val.callees.map(callee => ({ id: `c@${callee}`, type: "input", label: callee, position: POS }));
  const adts: Node[] = Object.keys(val.adts).map(adt => ({ id: `adt@${adt}`, type: "default", label: adt, position: POS }));
  // const adts_access: Node[] = Object.values(val.adts).flat().map(access => ({ id: `access@${access}`, type: "output", label: access, position: POS }));
  // const nodes = [root, ...callees, ...adts, ...adts_access];
  const nodes = [root, ...callees, ...adts];

  let edges: Edge[] = [];
  // Connect the root with leaves.
  callees.forEach(leaf => edges.push({ id: `e@${root.id}-${leaf.id}`, source: leaf.id, target: root.id, }));
  adts.forEach(leaf => edges.push({ id: `e@${root.id}-${leaf.id}`, source: root.id, target: leaf.id, }));
  // Connect adt and access.
  // for (const [adt, v_access] of Object.entries(val.adts)) {
  //   for (const access of v_access) {
  //     const id_adt = `adt@${adt}`;
  //     const id_access = `access@${access}`;
  //     edges.push({ id: `e@${id_adt}-${id_access}`, source: id_adt, target: id_access });
  //   }
  // }

  data.value = { nodes, edges };
})

/** Recompute node layout (position). */
async function layoutGraph(direction: string) {
  if (data.value.nodes.length === 0) return;
  data.value.nodes = layout(data.value.nodes, data.value.edges, direction)
  nextTick(fitView);
}

</script>
