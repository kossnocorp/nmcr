"""Generation output shared by CLI and MCP."""


from typing import Optional
from genotype import Model


class OutputFile(Model):
    path: Optional[str] = None
    """Optional relative path for this file in the output."""
    lang: Optional[str] = None
    """Optional language hint carried through for consumers."""
    content: str
    """Rendered file content."""


class OutputTree(Model):
    files: list[OutputFile]


type Output = OutputTree | OutputFile
