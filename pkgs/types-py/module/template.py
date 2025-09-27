"""Template schema: single-file and tree templates, plus a collection wrapper."""


from typing import Literal, Optional
from .arg import Arg
from .location import Location
from genotype import Model


class TemplateFile(Model):
    """A single-file template node."""

    kind: Literal["file"]
    """Discriminator for unions."""
    id: str
    name: str
    description: str
    args: list[Arg]
    lang: Optional[str] = None
    """Optional language hint from the fenced code block."""
    content: str
    """Raw template content."""
    path: Optional[str] = None
    """Optional relative path to use when writing to disk."""
    location: Location


class TemplateTree(Model):
    """A tree of template files grouped under a single heading."""

    kind: Literal["tree"]
    """Discriminator for unions."""
    id: str
    name: str
    description: str
    """Prose between the tree heading and the first file."""
    files: list[Template]
    location: Location


type Template = TemplateTree | TemplateFile
"""Union of templates."""


class TemplateCollection(Model):
    """A collection of top-level templates parsed from a single markdown file."""

    name: str
    description: str
    templates: list[Template]
    location: Location
