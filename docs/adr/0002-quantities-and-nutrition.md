# ADR 0002: quantities and nutrition

Status: Proposed implementation; customer unit preferences accepted
Date: 2026-11-07

## Context
Recipes accept varied input while the customer wants only grams/kilograms in personal views. Nutrition and educational effects require provenance.

## Decision
Retain exact decimal input and unit, dimension, source and conversion context. Normalize mass to grams, volume to millilitres and counts to pieces within their own dimensions. Convert across dimensions only with ingredient/state-specific density or piece mass. Keep serving scaling rational/decimal until display. Persist source values and conversion versions for reproducibility. Nutrition per serving includes units, reference basis, source, missingness and uncertainty. Educational component notes carry evidence and conditions.

## Alternatives
Storing everything as grams loses volume/count meaning and needs unavailable densities. Bare binary floating point and universal teaspoon conversions obscure precision. Preserve-only-without-normalization complicates aggregation.

## Consequences and open questions
Numeric library/scale, regional spoon standards, nutrient provider and cooking retention factors are unresolved. No conversion is fabricated. Display preferences never rewrite recipe source data.
