/** The way to view UPG. */
export enum ViewType {
  Callees = "Callees",
  Adts = "Adts",
}

export const ALL_VIEW_TYPES: ViewType[] = [
  ViewType.Callees, ViewType.Adts
];
