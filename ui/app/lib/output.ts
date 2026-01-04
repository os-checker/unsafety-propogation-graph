
export type Function = {
  name: string,
  safe: boolean,
  callees: string[],
  adts: { [key: string]: string[] },
  span: string,
  src: string,
  mir: string,
  doc: string,
}

export const EMPTY_FUNCTION: Function = {
  name: "", safe: true, callees: [], adts: {}, span: "",
  src: "", mir: "", doc: "",
};

