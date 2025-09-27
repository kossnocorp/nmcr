from genotype import Model
from typing import Literal


type ArgKind = Literal["any"] | Literal["boolean"] | Literal["string"] | Literal["number"]


class Arg(Model):
    name: str
    description: str
    kind: ArgKind
