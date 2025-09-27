from .span import Span
from genotype import Model


class Location(Model):
    path: str
    span: Span
