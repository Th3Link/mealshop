# Native shopping interaction contract

The screen's job is finding the next item and marking it purchased. The customer-supplied Bring image
informs compact item tiles, an obvious search/entry point and separated purchased goods. Rebuild these
patterns with native Nextcloud checkboxes, buttons, dialog, colors and fonts. Use the existing MDI family;
icons are decorative category cues from explicit linked ingredient metadata, never dietary declarations.
An unlinked item has the generic basket icon. Quantities remain exact and visibly qualified by unit.

Keep tile labels wrapping and the whole checkbox area tappable. Use at least 44px controls and 8px gaps.
Move provenance and shop controls into a named dialog, leaving a separate, labelled details action on
each tile. Keep details reachable in shopping mode. A search is a cached derived view; it never changes
shopping data or order. Clear the search before reordering a shop so hidden items cannot be dropped.

Search sits above the tiles. Its add action pre-fills only the item label and reveals a compact inline
form with the existing explicit quantity/unit fields and optional confirmed ingredient link. Enter and
Escape work without throwing away a changed draft. Successful add clears search, restores its focus
and shows the saved item; failures retain input. A last successful check/uncheck exposes Undo. Checking
advances keyboard focus to the next visible open item, or the purchased disclosure; unchecking reveals
the restored item. Do not optimistically claim purchases before the server accepts them.

UX-UI-Pro-Max's verified touch-spacing and Vue computed-state guidance applies. Taste informs equal
edges, concise hierarchy, icon consistency and screenshot refinement. Its marketing layout, palette,
font, animation and standalone checker requirements are overridden by the customer's native Nextcloud
hierarchy and real embedded browser acceptance. No marketing kit or global reset is introduced.
