export interface Template {
  id: string;
  name: string;
  description: string;
  args: Array<import("./arg.js").Arg>;
  lang?: string | undefined;
  content: string;
  location: import("./location.js").Location;
}

export interface TemplateCollection {
  name: string;
  description: string;
  templates: Array<Template>;
  location: import("./location.js").Location;
}
