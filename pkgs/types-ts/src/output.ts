/** @file Generation output shared by CLI and MCP. */

export interface OutputFile {
  /** Optional relative path for this file in the output. */
  path?: string | undefined;
  /** Optional language hint carried through for consumers. */
  lang?: string | undefined;
  /** Rendered file content. */
  content: string;
}

export interface OutputTree {
  files: Array<OutputFile>;
}

export type Output = OutputTree | OutputFile;
