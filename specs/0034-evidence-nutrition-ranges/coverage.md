# Public coverage and source selection

The original 313 ingredient identities and their reported maps remain unchanged. This audit reads
public repository facts only. Private audit findings are not part of this published coverage report.

| Nutrient | Reported (including same-record supplements) | Estimated | Unknown |
|---|---:|---:|---:|
| `energy_kcal` | 284 | 29 | 0 |
| `protein` | 282 | 31 | 0 |
| `fat` | 283 | 30 | 0 |
| `available_carbohydrate` | 248 | 28 | 37 |
| `carbohydrate` | 36 | 9 | 268 |
| `fibre` | 270 | 28 | 15 |
| `total_sugars` | 270 | 29 | 14 |
| `saturated_fat` | 283 | 29 | 1 |
| `salt` | 248 | 28 | 37 |
| `calcium` | 283 | 20 | 10 |
| `iron` | 282 | 22 | 9 |
| `magnesium` | 283 | 20 | 10 |
| `potassium` | 283 | 20 | 10 |
| `sodium` | 278 | 20 | 15 |
| `vitamin_c` | 210 | 16 | 87 |
| `food_folate` | 260 | 20 | 33 |
| `folate` | 34 | 8 | 271 |

Each food has energy, protein, fat and at least one correctly defined carbohydrate value.
The two carbohydrate definitions and the two folate definitions remain separate; their unknown
counts must not be interpreted as omitted ingredients or summed across definitions.
Unsupported individual nutrients stay unknown, with explicit partial totals in recipes.

## Comparative selections

These are observed spans of selected records, not guaranteed bounds or confidence intervals.
Each selection retains its precise rationale in the frozen evidence file and the native UI.

| Ingredient | Comparison records |
|---|---|
| `psyllium-husks` | [dmBio · Psyllium husks](https://www.dm.de/p/d/1691049/dmbio-flohsamenschalen); [Mivolis · Ground psyllium husks](https://www.dm.de/p/d/3104212/mivolis-flohsamenschalen-gemahlen) |
| `erythritol` | [KoRo · Pure erythritol](https://www.koro.com/lude/erythrit-1-kg); [EDEKA · Pure erythritol](https://www.edeka.de/sortiment/4311501132494/) |
| `quark-double-cream-unspecified` | [Quark 50 % fat in dry matter](https://blsdb.de/download); [Mascarpone min. 80 % fat in dry matter](https://blsdb.de/download) |
| `tomatoes-chopped-canned-unspecified` | [Tomato peeled, canned](https://blsdb.de/download); [Tomato canned](https://blsdb.de/download) |
| `pasta-durum-wheat-unspecified` | [Pasta egg-free, raw](https://blsdb.de/download); [Spelt semolina](https://blsdb.de/download) |
| `pasta-lentil-dry-unspecified` | [dmBio · Red lentil fusilli](https://www.dm.de/p/d/1517830/dmbio-nudeln-fusilli-aus-roten-linsen); [Lentil flour](https://blsdb.de/download) |
| `pasta-chickpea-dry-unspecified` | [dmBio · Chickpea fusilli](https://www.dm.de/p/d/1517827/dmbio-nudeln-fusilli-aus-kichererbsen); [Chickpea flour](https://blsdb.de/download) |
| `pasta-pea-dry-unspecified` | [Pea flour](https://blsdb.de/download); [dmBio · Chickpea fusilli](https://www.dm.de/p/d/1517827/dmbio-nudeln-fusilli-aus-kichererbsen) |
| `pasta-spelt-dry-unspecified` | [Spelt flour, Type 630](https://blsdb.de/download); [Spelt semolina](https://blsdb.de/download) |
| `pasta-wholemeal-spelt-dry-unspecified` | [Spelt wholemeal flour](https://blsdb.de/download); [Spelt coarse meal](https://blsdb.de/download) |
| `pasta-rice-maize-dry-unspecified` | [Pasta egg-free, gluten-free, raw](https://blsdb.de/download); [Rice flour](https://blsdb.de/download) |
| `rice-paper-unspecified` | [Rice flour](https://blsdb.de/download); [Rice noodles, dry](https://fdc.nal.usda.gov/food-details/169742/nutrients) |
| `pea-protein-isolate-unspecified` | [KoRo · Organic pea protein isolate](https://www.korodrogerie.de/bio-erbsenprotein-500-g); [Raab Vitalfood · Organic pea protein](https://www.raabvitalfood.de/products/bio-erbsenprotein-pulver) |
| `pea-protein-concentrate-unspecified` | [Organic Workout · Organic pea protein concentrate](https://organic-workout.de/products/bio-premium-erbsenprotein-500-g); [Raab Vitalfood · Organic pea protein](https://www.raabvitalfood.de/products/bio-erbsenprotein-pulver) |
| `whey-protein-concentrate-unspecified` | [KoRo · Chocolate whey concentrate/isolate blend](https://www.korodrogerie.de/whey-protein-chocolate-1-kg); [KoRo · Vanilla whey concentrate/isolate blend](https://www.korodrogerie.de/whey-protein-vanilla-style-1-kg); [Beverages, Protein powder whey based](https://fdc.nal.usda.gov/food-details/173180/nutrients) |
| `whey-protein-isolate-unspecified` | [Myprotein · Impact Whey Isolate, vanilla](https://de.myprotein.com/p/sporternahrung/impact-whey-isolate/10530911/); [Beverages, Whey protein powder isolate](https://fdc.nal.usda.gov/food-details/173177/nutrients) |
| `casein-protein-unspecified` | [Bulk · Micellar casein, displayed reference flavour](https://www.bulk.com/de/products/mizellares-casein-de/bpb-mpi9-0000); [Myprotein · Slow-Release Casein, unflavoured](https://de.myprotein.com/p/sporternahrung/slow-release-casein/10798909/) |
| `rice-protein-unspecified` | [Raab Vitalfood · Organic rice protein](https://www.raabvitalfood.de/products/bio-reisprotein-pulver); [Bulk · Brown rice protein](https://www.bulk.com/de/products/naturreis-protein-80/bpb-brp8-0000) |
| `hemp-protein-unspecified` | [Raab Vitalfood · Partially defatted hemp protein](https://www.raabvitalfood.de/products/naturland-bio-hanfprotein-pulver); [KoRo · Partially defatted hemp protein](https://www.korodrogerie.de/teilentoeltes-bio-hanfprotein-1-kg) |
| `vegan-protein-blend-unspecified` | [KoRo · Neutral vegan protein blend](https://www.korodrogerie.de/vegan-protein-powder-neutral-1-kg); [KoRo · Vanilla vegan protein blend](https://www.korodrogerie.de/veganes-protein-pulver-vanille-1-kg) |
| `protein-pudding-unspecified` | [Zott · Chocolate protein pudding](https://www.zott-dairy.com/de/marken-produkte/zott-protein/pudding/); [Müller · Vanilla high protein pudding](https://allesmuelleroderwas.de/produkte/high-protein/high-protein-pudding/high-protein-pudding-vanille-geschmack) |
| `protein-yogurt-unspecified` | [Skyr Icelandic style strained yogurt < 10 % fat in dry matter](https://blsdb.de/download); [Yogurt, Greek, plain, nonfat (Includes foods for USDA's Food Distribution Program)](https://fdc.nal.usda.gov/food-details/170894/nutrients); [Yogurt, Greek, vanilla, nonfat](https://fdc.nal.usda.gov/food-details/170902/nutrients) |
| `protein-bar-unspecified` | [Formulated Bar, SOUTH BEACH protein bar](https://fdc.nal.usda.gov/food-details/173158/nutrients); [Formulated bar, MARS SNACKFOOD US, SNICKERS MARATHON Protein Performance Bar, Caramel Nut Rush](https://fdc.nal.usda.gov/food-details/174786/nutrients) |
| `tofu-smoked-unspecified` | [Taifun · Smoked tofu classic](https://www.taifun-tofu.de/de/produkte/raeuchertofu-klassik); [Tofu](https://blsdb.de/download) |
| `tofu-silken-unspecified` | [Silken tofu](https://blsdb.de/download); [MORI-NU, Tofu, silken, soft](https://fdc.nal.usda.gov/food-details/174292/nutrients); [MORI-NU, Tofu, silken, firm](https://fdc.nal.usda.gov/food-details/172461/nutrients) |
| `parsley-root-unspecified` | [Parsley root raw](https://blsdb.de/download); [Parsnip raw](https://blsdb.de/download) |
| `lovage-fresh-unspecified` | [Basil raw](https://blsdb.de/download); [Parsley leaf raw](https://blsdb.de/download); [Thyme, fresh](https://fdc.nal.usda.gov/food-details/173470/nutrients); [Rosemary, fresh](https://fdc.nal.usda.gov/food-details/173473/nutrients) |
| `oregano-fresh-unspecified` | [Basil raw](https://blsdb.de/download); [Parsley leaf raw](https://blsdb.de/download); [Thyme, fresh](https://fdc.nal.usda.gov/food-details/173470/nutrients); [Rosemary, fresh](https://fdc.nal.usda.gov/food-details/173473/nutrients) |
| `sage-fresh-unspecified` | [Basil raw](https://blsdb.de/download); [Parsley leaf raw](https://blsdb.de/download); [Thyme, fresh](https://fdc.nal.usda.gov/food-details/173470/nutrients); [Rosemary, fresh](https://fdc.nal.usda.gov/food-details/173473/nutrients) |
| `xylitol-powder` | [Sugar white (refined sugar)](https://blsdb.de/download); [Sugar brown (candyfarin/raw sugar)](https://blsdb.de/download) |
| `coconut-oil` | [Coconut oil deodorized](https://blsdb.de/download); [Sunflower oil](https://blsdb.de/download) |
| `balsamic-vinegar` | [Wine vinegar](https://blsdb.de/download); [Apple cider vinegar](https://blsdb.de/download) |

## Boundaries and review

- Group spans use only explicit members, including their explicitly named states. They do not
  convert raw to cooked mass or imply equivalent states; choose a subtype for a relevant estimate.
- Never infer allergy safety, beta-glucan or soluble fibre from comparable composition.
- Censored `< x` declarations retain an enclosing 0…x bound, not a zero reading.
- Excluded: inconsistent `KoRo` rice-protein energy declarations and per-100-ml pudding tables
  without density evidence. See the exclusions and matching rationale in the frozen evidence file.
- The 32 comparative food profiles and 73 group spans coexist with same-source reported supplements.
- Legacy recipe reads add immutable-v1 analysis in memory; new revisions pin effective evidence.

Reproduce: `python3 scripts/import_nutrition_estimates.py --check` and
`python3 scripts/verify_nutrition_estimates.py`. Original archives were independently checked with
`python3 scripts/verify_nutrition_archives.py --bls /tmp/mealshop-bls.zip --usda /tmp/mealshop-usda-sr.zip`
(both SHA-256 hashes and 1,341 raw facts/acquisition metadata). The normal checks need no network.
