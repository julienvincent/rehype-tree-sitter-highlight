;; extends

((list_lit
  ((sym_lit) @def-type
   (sym_lit) @def-name
   (str_lit)? @docstring @injection.content)
   (map_lit)?

   [
    (vec_lit)
    (list_lit (vec_lit))+
   ])

  (#match? @def-type "^(defn-?|defmacro)$")
  (#offset! @injection.content 0 1 0 -1)
  (#set! injection.language "markdown"))
