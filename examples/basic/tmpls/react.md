# React

React templates collection.

## React component

React component file template with optional props.

### Args

- `props?` [boolean]: Include a props interface scaffold.
- `name` [string]: Component name.

### Template

```tsx
import React from "react";

//$if props
export namespace $Name$ {
  export interface Props {
    // $1 -- Props here
  }
}
//$end

export function $Name$(/*$if props*/ props: $Name$.Props /*$end*/) {
  const {
    /* $2 -- Destructure props here */
  } = props;
  return <div>{/* $3 -- Content here */}</div>;
}
```
