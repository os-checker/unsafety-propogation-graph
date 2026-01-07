
export type Function = {
  name: string,
  safe: boolean,
  callees: string[],
  adts: { [key: string]: string[] },
  span: string,
  src: string,
  mir: string,
  doc: string,
  tags: Tags
}

export type Tags = {
  tags: Property[],
  spec: { [key: string]: TagSpec },
  docs: string[],
}

export type Property = {
  tag: { name: string, typ: TagType | null },
  args: string,
}

export type TagSpec = {
  args: string[],
  desc: string | null,
  expr: string | null,
  types: TagType[],
  url: string | null,
}

export enum TagType {
  Precond = "precond",
  Hazard = "hazard",
  Option = "option",
}

export const EMPTY_FUNCTION: Function = {
  name: "", safe: true, callees: [], adts: {}, span: "",
  src: "", mir: "", doc: "", tags: { tags: [], spec: {}, docs: [] },
};

