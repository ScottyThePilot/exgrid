# ExGrid

A chunk-based implementation of a 2D collection.
The positions associated with values are integer coordinates on a euclidean plane, including negative values.

Serde support is largely limited by what format you use, as most plaintext formats don't
seem to support non-string keys in maps, which is how serialized `ExGrid`s are represented.
