<template>
  <div class="top-menu">
    <UNavigationMenu :items="navi_menu" class="w-1/2" trailing-icon="tabler:chevron-right">
      <template #item-content="{ index: stack_idx }">
        <div class="flex gap-4 m-2">
          <div v-for="[kind, v_sub_navi_idx] in Object.entries(currentNaviItem(stack_idx)?.groups ?? {})" :key="kind">
            <div :class="[colorClass(kind), 'text-center font-bold']">{{ kind }}</div>
            <div>
              <ul @click="(event) => naviItemClick(event, stack_idx)">
                <li v-for="{ sub_navi_idx, item } in v_sub_navi_idx.map(
                  idx => ({ sub_navi_idx: idx, item: currentNaviItem(stack_idx)?.subitems[idx] })
                )" :data-idx="item?.idx" :data-sub-navi-idx="sub_navi_idx" class="my-1">
                  <UButton :label="item?.name ?? 'ERROR-NAME'" :icon="icon(kind)" size="md" color="neutral"
                    variant="ghost" class="w-full" />
                </li>
              </ul>
            </div>
          </div>
        </div>
      </template>
    </UNavigationMenu>

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
import { ViewType, ALL_VIEW_TYPES, EMPTY_NAVI, NAVI_URL, type Navigation, icon, colorClass, DefPathKind, type NaviItem } from '~/lib/topbar';

const viewSelected = defineModel<ViewType[]>('viewSelected');

const views = ref<ViewType[]>(ALL_VIEW_TYPES);

const navi = ref<Navigation>(EMPTY_NAVI);
$fetch(NAVI_URL)
  .then(text => navi.value = JSON.parse(text as string))
  .catch(err => console.log(err));

// Expanded navi items. The value is data idx in Navigation.
const navi_stack = ref<number[]>([]);
function currentNaviItem(stack_idx: number): NaviItem | undefined {
  const idx = navi_stack.value[stack_idx];
  if (idx === undefined) return undefined;
  return navi.value.navi[idx]
}

const navi_menu = ref<NavigationMenuItem[]>([]);
watch(navi, val => {
  const data = val.data;
  const nav = val.navi;
  const root = data[0]?.[0];
  if (!root) {
    navi_menu.value = [];
    navi_stack.value = [];
    return;
  }

  const tree: NavigationMenuItem[] = [{
    label: root.name, icon: icon(root.kind),
    // children: nav[0]?.subitems?.map(item => ({ label: item.name, icon: icon(item.kind) })) ?? []
  }];
  navi_menu.value = tree;
  navi_stack.value.push(0);
});

/** Respond to which navi item is clicked.
* stack_idx refers to the current position in navi_stack.
* li id refers to navi data idx.
*/
function naviItemClick(event: MouseEvent, stack_idx: number) {
  const li = (event.target as HTMLElement).closest('li')
  if (li && (event.currentTarget as HTMLElement).contains(li)) {
    const idx = parseInt(li.dataset.idx ?? "");
    const sub_navi_idx = parseInt(li.dataset.subNaviIdx ?? "");

    // This is never null, because we just clicked it.
    // const clicked = {
    //   full_path: navi.value.data[idx]!,
    //   short: navi.value.navi[stack_idx]?.subitems[sub_navi_idx]!
    // };
    const clicked = navi.value.navi[stack_idx]?.subitems[sub_navi_idx]!;

    // This can be null when fn item is clicked or the item has no sub items.
    // const target = navi.value.navi[idx]?.subitems;
    // console.log("\nstack_idx:", stack_idx, "\nsub_navi_idx:", sub_navi_idx, "\nclicked:", clicked, "\ntarget:", target);

    const clicked_kind = clicked.kind;
    if (clicked_kind !== DefPathKind.Fn && clicked_kind !== DefPathKind.AssocFn) {
      navi_stack.value.push(clicked.idx);
      navi_menu.value.push({
        label: clicked.name,
        icon: icon(clicked.kind),
        // children: target?.map(item => ({ label: item.name, icon: icon(item.kind) })) ?? []
      })
    }
  }
}
</script>
