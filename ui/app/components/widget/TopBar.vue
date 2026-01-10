<template>
  <div class="top-menu">
    <UNavigationMenu :items="navi_menu" class="w-1/2" trailing-icon="tabler:chevron-right" />

    <div class="top-menu mr-2 gap-4">
      <div>
        Graph View:
        <USelectMenu v-model="viewSelected" multiple :items="views" :search-input="false" class="w-50" />
      </div>
      <UColorModeButton />
      <!-- <ULink to="https://artisan-lab.github.io/RAPx-Book/6.4-unsafe.html" :external="true" target="_blank">Help</ULink> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui';
import { ViewType, ALL_VIEW_TYPES, EMPTY_NAVI, NAVI_URL, type Navigation, icon } from '~/lib/topbar';

const viewSelected = defineModel<ViewType[]>('viewSelected');

const views = ref<ViewType[]>(ALL_VIEW_TYPES);

const navi = ref<Navigation>(EMPTY_NAVI);
$fetch(NAVI_URL)
  .then(text => navi.value = JSON.parse(text as string))
  .catch(err => console.log(err));

const navi_menu = computed<NavigationMenuItem[]>(() => {
  const data = navi.value.data;
  const nav = navi.value.navi;
  const root = data[0]?.[0];
  if (!root) return [];

  let children = nav[0]!.map(item => ({ label: item.name, icon: icon(item.kind) }));
  let tree: NavigationMenuItem[] = [
    {
      label: root.name, icon: icon(root.kind), children
    },
    // ...children
  ];

  // nav[0]!.map(item => tree.push({ label: item.name }))

  console.log(tree)
  return tree
});
</script>

<style lang="css">
.top-menu {
  @apply flex items-center justify-between;
}
</style>
