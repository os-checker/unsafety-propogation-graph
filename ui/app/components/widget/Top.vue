<template>
  <div class="top-menu">
    <UNavigationMenu :items="menuItems" />

    <div>
      Graph View:
      <USelectMenu v-model="viewSelected" multiple :items="views" :search-input="false" class="w-50" />
    </div>

    <div class="top-menu mr-2">
      <UColorModeButton />
      <!-- <ULink to="https://artisan-lab.github.io/RAPx-Book/6.4-unsafe.html" :external="true" target="_blank">Help</ULink> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui';
import { ViewType, ALL_VIEW_TYPES } from '~/lib/topbar';

const menuItems = ref<NavigationMenuItem[]>([{ label: "function", }]);

const viewSelected = defineModel<ViewType[]>('viewSelected');

const views = ref<ViewType[]>(ALL_VIEW_TYPES);
</script>

<style lang="css">
.top-menu {
  @apply flex items-center justify-between;
}
</style>
