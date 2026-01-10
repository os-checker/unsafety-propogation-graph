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
  Mod = "Mod",
  Fn = "Fn",
  AssocFn = "AssocFn",
  Struct = "Struct",
  Enum = "Enum",
  Union = "Union",
  TraitDecl = "TraitDecl",
  SelfTy = "SelfTy",
  ImplTrait = "ImplTrait",
}

/** Returns an icon string for a DefPathKind.
 * The icon must be maintained in nuxt config.*/
export function icon(kind: DefPathKind): string {
  switch (kind) {
    case DefPathKind.Mod: return "tabler:letter-m";
    case DefPathKind.Fn: return "tabler:square-letter-f";
    case DefPathKind.AssocFn: return "tabler:square-letter-f";
    case DefPathKind.Struct: return "tabler:letter-s";
    case DefPathKind.Enum: return "tabler:letter-e";
    case DefPathKind.Union: return "tabler:letter-u";
    case DefPathKind.TraitDecl: return "tabler:letter-t";
    case DefPathKind.SelfTy: return "tabler:letter-t-small";
    case DefPathKind.ImplTrait: return "tabler:letter-t";
    default: return "tabler:alert-circle";
  }
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
