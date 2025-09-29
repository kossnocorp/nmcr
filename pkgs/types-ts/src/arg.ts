export interface Arg {
  name: string;
  description: string;
  kind: ArgKind;
  required: boolean;
}

export type ArgKind = "any" | "boolean" | "string" | "number";
