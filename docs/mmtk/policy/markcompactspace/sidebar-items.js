initSidebarItems({"constant":[["GC_EXTRA_HEADER_BYTES",""],["GC_EXTRA_HEADER_WORD","For each MarkCompact object, we need one extra word for storing forwarding pointer (Lisp-2 implementation). Note that considering the object alignment, we may end up allocating/reserving more than one word per object. See [`MarkCompactSpace::HEADER_RESERVED_IN_BYTES`]."],["GC_MARK_BIT_MASK",""]],"struct":[["MarkCompactObjectSize",""],["MarkCompactSpace",""]]});