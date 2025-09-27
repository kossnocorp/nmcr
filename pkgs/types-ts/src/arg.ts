export interface Arg {
  name: string;
  description: string;
  kind: ArgKind;
}

export type ArgKind = "any" | "boolean" | "string" | "number";
