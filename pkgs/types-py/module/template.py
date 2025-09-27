from .arg import Arg
from typing import Optional
from .location import Location
from genotype import Model


class Template(Model):
    id: str
    name: str
    description: str
    args: list[Arg]
    lang: Optional[str] = None
    content: str
    location: Location


class TemplateCollection(Model):
    name: str
    description: str
    templates: list[Template]
    location: Location
