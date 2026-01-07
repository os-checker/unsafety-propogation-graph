<template>
  <div class="top-menu">
    <UNavigationMenu :items="items" />

    <div class="top-menu mr-2">
      <UColorModeButton />
      <!-- <ULink to="https://artisan-lab.github.io/RAPx-Book/6.4-unsafe.html" :external="true" target="_blank">Help</ULink> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui';

const items = ref<NavigationMenuItem[]>([{ label: "function", }]);
</script>

<style lang="css">
.top-menu {
  @apply flex items-center justify-between;
}
</style>
