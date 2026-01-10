/** The way to view UPG. */
export enum ViewType {
  Callees = "Callees",
  Adts = "Adts",
}

export const ALL_VIEW_TYPES: ViewType[] = [
  ViewType.Callees, ViewType.Adts
];

// Navigation

export enum DefPathKind {
  Mod,
  Fn,
  AssocFn,
  Struct,
  Enum,
  Union,
  TraitDecl,
  SelfTy,
  ImplTrait,
}
export type DefPath = {
  kind: DefPathKind,
  name: string,
}
export type ItemPath = DefPath[];
export type NaviItem = {
  idx: number, name: string, kind: DefPathKind,
}
export type Navigation = {
  data: ItemPath[],
  navi: { [key: number]: NaviItem[] },
  name_to_path: { [key: string]: number },
  path_to_name: { [key: number]: string },
}

export const EMPTY_NAVI: Navigation = { data: [], navi: {}, name_to_path: {}, path_to_name: {} };
export const NAVI_URL = "https://raw.githubusercontent.com/os-checker/unsafety-propagation-graph-data/refs/heads/main/test/poc/navi/navi.json";
