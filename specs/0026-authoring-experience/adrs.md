# Decisions

Status: Accepted

Extend recipe records with optional stable line IDs and part IDs, named recipe-owned parts, and step references. Keep the flattened ingredient lines as the single calculation/shopping source; parts organize them without duplicating amounts. A part may use earlier prepared parts via an acyclic dependency list. Imported/independently versioned subrecipe references are outside this customer example and not silently assumed.

Keep beta-glucan and soluble fibre serialized under their existing stable vocabulary for compatibility, but expose them through a distinct optional-constituent category and result section. Missing constituent values stay unknown. This avoids rewriting frozen source records or old recipes.

Sorbitol is not added to the regulated fourteen-allergen source mapping. A separate trigger registry and explicit household declarations feed the existing conservative constraint review. Source: [NHS](https://www.nhs.uk/conditions/irritable-bowel-syndrome-ibs/diet-lifestyle-and-medicines/) (classification context only; no medical recommendation in the product).
