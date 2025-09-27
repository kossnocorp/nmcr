/** @file Template schema: single-file and tree templates, plus a collection wrapper. */

/** A single-file template node. */
export interface TemplateFile {
  /** Discriminator for unions. */
  kind: "file";
  id: string;
  name: string;
  description: string;
  args: Array<import("./arg.js").Arg>;
  /** Optional language hint from the fenced code block. */
  lang?: string | undefined;
  /** Raw template content. */
  content: string;
  /** Optional relative path to use when writing to disk. */
  path?: string | undefined;
  location: import("./location.js").Location;
}

/** A tree of template files grouped under a single heading. */
export interface TemplateTree {
  /** Discriminator for unions. */
  kind: "tree";
  id: string;
  name: string;
  /** Prose between the tree heading and the first file. */
  description: string;
  files: Array<Template>;
  location: import("./location.js").Location;
}

/** Union of templates. */
export type Template = TemplateTree | TemplateFile;

/** A collection of top-level templates parsed from a single markdown file. */
export interface TemplateCollection {
  name: string;
  description: string;
  templates: Array<Template>;
  location: import("./location.js").Location;
}
