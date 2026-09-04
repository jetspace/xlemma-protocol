import Lean

namespace XLemma

/--
Prototype export marker. The external packer captures the declaration output,
then a production environment exporter serializes the elaborated expression.
-/
macro "#xlemma_export " declaration:ident : command => `(#print $declaration)

end XLemma
