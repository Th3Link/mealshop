# Brewing-method and equipment research

Date: 2026-11-13. Origin: the customer's explicit request to research BIAB, Grainfather, Brew Monk,
Speidel Braumeister, DIY all-grain, extract, partial mash and related variants before F14 implementation.
This document separates source facts from Mealshop's proposed representation. No manufacturer cloud
integration, controller compatibility, universal recipe, efficiency or safety result is inferred.

## Process families

| Family | What changes in the process | Consequence for a usable template | Primary source |
| --- | --- | --- | --- |
| Malt extract (LME/DME) | Concentrated malt extract supplies the wort's fermentable material; the homebrew process need not recreate its original mash. | Dissolve/add extract, recipe-specific boil/additions, cool and ferment; retain liquid/dry extract identity. | [AHA: extract manufacture](https://www.homebrewersassociation.org/how-to-brew/how-malt-extract-is-made/) |
| Extract with specialty-grain steeping | Grain steeping adds recipe character; it is distinct from relying on a conversion mash. | Add a separately named steep/remove-grain phase before extract additions. Do not call it partial mash. | [AHA: working with extract recipes](https://www.homebrewersassociation.org/how-to-brew/working-with-extract-recipes/) |
| Partial mash / mini mash | Some fermentables come from grain mashing; extract supplies another portion. | Include a real mash and grain separation as well as explicit extract additions; keep both ingredient sources. | [AHA: partial mash](https://homebrewersassociation.org/tutorials/partial-mash-homebrewing/partial-mash-homebrewing/) |
| All-grain | The recipe uses a grain mash to produce wort before boiling and fermentation. | Include mash, lautering, optional sparge and collection; equipment determines transfers. | [AHA: batch-sparge all-grain](https://homebrewersassociation.org/tutorials/all-grain-batch-sparge-homebrewing/all-grain-batch-sparge-homebrewing/) |
| Brew in a bag (BIAB) | An all-grain implementation using a removable bag as the grain separator, commonly in one vessel. | Mash in the bag, lift/drain, then boil in the kettle. Optional rinse is an explicit variant, not an invisible mandatory step. | [AHA: BIAB](https://www.homebrewersassociation.org/wp-content/uploads/How-To-Brew-in-a-Bag.pdf) |

A concentrated/partial-volume boil with later top-up is a separate volume choice from partial **mash**.
Extract additions may occur at more than one stage. Conversion between extract and grain requires
recipe and equipment assumptions; it is not a universal substitution factor. These distinctions are
supported by [AHA's extract discussion](https://www.homebrewersassociation.org/how-to-brew/working-with-extract-recipes/).
Pre-hopped kits are also not assumed to require the same hop-boil sequence as unhopped extract;
kit-specific instructions remain necessary rather than inventing a universal no-boil recipe.

## Equipment families and the named systems

| Layout | Observed workflow distinction | Equipment/profile data Mealshop needs | Source |
| --- | --- | --- | --- |
| Bag + kettle (DIY BIAB) | The bag separates grain; there need not be a separate mash/lauter vessel. | Kettle and bag identity, user-declared working capacity, lift/drain phase, chosen water/boil plan. | [AHA BIAB](https://www.homebrewersassociation.org/wp-content/uploads/How-To-Brew-in-a-Bag.pdf) |
| Grainfather G30 example | A removable grain basket, top-return recirculation, basket lifting/draining and separately described sparging. | Actual model/generation, vessel/pump resource, basket-removal phase, optional sparge-water vessel and actual chiller. | [Manufacturer G30 English manual](https://help.grainfather.com/hc/en-us/article_attachments/15548590179345), pp. 7–10; [manual index](https://help.grainfather.com/hc/en-us/articles/360011864078-Instructions-G30) |
| Brew Monk B40/B50/B70 example | Malt pipe/filter plates, circulation pipe and pump with adjustable return; hardware and operating modes have their own instructions. | Actual model, vessel/pump resource, malt-pipe handling and independently declared settings. | [Brouwland manufacturer/distributor manual](https://brouwland.com/brldocs/057.702.xx_BrewMonk_Manual.pdf), English section pp. 18–25 |
| Speidel Braumeister PLUS example | Wort rises through the malt pipe and overflows; pump pauses and malt-pipe removal belong to this system's process. | Distinct upward-flow profile, actual model, malt-pipe release/removal and available cooling equipment. | [Speidel original English manual](https://www.speidels-braumeister.de/en/service/instruction-manuals/original-instruction-manual-speidels-braumeister-plus-10-20-50-100-litres.html), §§6.4–6.6, manual dated September 2020 |
| Cooler mash/lauter tun + kettle | A separate insulated mash/lauter vessel retains grain; wort is collected into the boiling vessel. | Mash tun, kettle and, if used, sparge-water vessel; transfers and actual heating plan. | [AHA all-grain tutorial](https://homebrewersassociation.org/tutorials/all-grain-batch-sparge-homebrewing/all-grain-batch-sparge-homebrewing/) |
| Two-vessel recirculating setup | Vessel count alone does not define the heat/recirculation path. Two-vessel kettle-RIMS is one concrete implementation. | Separate resource IDs and explicit circulation/heat method instead of assuming a three-vessel layout. | [Blichmann BrewEasy](https://www.blichmannengineering.com/product/breweasy-compact-all-grain-brewing-system/) |
| Three-vessel DIY | Hot-liquor tank, mash/lauter tun and boil kettle have distinct roles. | Separate reservations and connections; the same phase can need several resources. | [The Electric Brewery's own three-vessel design](https://shop.theelectricbrewery.com/pages/kettles-overview) |
| HERMS | Recirculating wort receives heat through a heat exchanger in the hot-liquor tank. | Mash vessel, pump and HLT/heat-exchanger resources; recipe-specific rest/ramp entries. | [Builder's HLT/HERMS design](https://shop.theelectricbrewery.com/pages/hot-liquor-tank) |
| RIMS | Recirculating wort passes a direct heat source in the loop. | Explicit loop/pump/heater resource identities and the actual instrument's documentation. | [Blichmann controller manual, sensor placement](https://www.blichmannengineering.com/wp-content/uploads/UpdatedManuals/Electric_TOP_240V_Controller_Manual_V3_0.pdf); [builder's HERMS/RIMS comparison](https://shop.theelectricbrewery.com/pages/faq-herms-vs-rims) |

Brand names are examples of equipment families, not separate fermentation chemistries or claims of
remote control support. G30 generations and other Grainfather models have separate manuals. The app
must not copy one model's capacities, plumbing or chiller assumptions onto every device with the same
brand. DIY profiles need editable resources; a brand preset cannot describe every custom build.

## Independent choices

- **Mash programme:** single infusion, multiple rests/step mash or decoction. Decoction removes a
  portion to another vessel and returns it; that transfer/resource must be representable. This is
  independent of whether the wort is later batch- or fly-sparged. [AHA: decoction](https://www.homebrewersassociation.org/zymurgy-article/decoction-mashing-history-and-modern-approaches/)
- **Lautering/sparging:** no sparge; one or more batch rinses after drainage; or continuous/fly sparge
  while collecting wort. Preserve the chosen water volumes and repeated phases. Do not claim that a
  particular efficiency always follows from a technique. [AHA: sparging comparison](https://www.homebrewersassociation.org/how-to-brew/fly-sparing-vs-batch-sparging/)
- **Heating:** insulated/manual, direct heat, HERMS or RIMS. Record the user's actual configuration;
  the software schedules actions and records readings, not electrical or pump control.
- **Boil/volume:** full volume versus concentrated boil/top-up, with extract and hop additions attached
  to explicit phases. No automatic water, bitterness or extract conversions are inferred.
- **Cooling and fermentation:** independently assigned equipment and editable phases. Elapsed time is
  an observation/reminder, not a declaration that fermentation or packaging is complete.

## Implementation interpretation

The following is Mealshop's design derived from the researched distinctions, not a manufacturer's API:

1. Store fermentable strategy, vessel layout, heating and sparging as independent typed choices.
   Offer understandable starter templates for the named systems and DIY layouts without a combinatorial
   list of nearly identical recipes. BIAB remains identifiable within all-grain.
2. Keep user brew recipes and immutable batch snapshots separate from food-recipe servings/nutrition.
   A batch has an exact target volume and its own measured ingredients, phases and equipment bindings.
3. Reuse the durable timer/timezone/reminder engine. Preview actual dates, including an optional explicit
   yeast-preparation reminder before brew day; never prescribe one universal yeast handling schedule.
4. Record raw readings with original units, measurement timestamp, manual/device attribution and known
   or unknown calibration. Do not silently equate SG, Plato and Brix or temperature-correct them.
5. Check actual equipment reservations across batches and keep their operation/reminder identities
   independent. Editing one batch must not reset another's timers or original readings.
6. Cider, white wine, red wine and traditional mead have independently reviewed templates below.
   Their typed processes contain no grain/layout/heat/sparge classifications.

## Source use and limits

Sources were read online on 2026-11-13. URLs and the identified manual/model are retained above;
manufacturer content remains copyrighted. The implementation uses original phase labels/brief prompts
and links, not reproduced manuals, diagrams, recipe text or example temperature/time programmes.
Older manuals' legal, nutritional and troubleshooting claims are not adopted as current application
rules. Model capacities and recipes must remain explicitly supplied/reviewed facts. No comparative
product recommendation or claim that one brewing method is superior is made here.

## Measurement and yeast handling references

Original instrument calibration is part of the observation, not an application-wide correction.
The [iSpindel calibration documentation](https://www.ispindel.de/docs/Calibration_en.html) describes
instrument-specific calibration; [Tilt's manufacturer FAQ](https://tilthydrometer.com/pages/faqs)
describes its device readings and calibration. The model therefore retains the raw unit, source,
calibration declaration and separate measurement/entry times, without assuming a configured connector.
[Fermentis' product-specific usage guidance](https://fermentis.com/en/easy-2-use/) is another reason
not to impose a universal day-before yeast treatment. F14 offers an explicitly authored recipe reminder,
resolved by local calendar day, instead. These are primary-reference-informed implementation decisions;
no numeric beer chemistry, device credential contract or food-safety conclusion is inferred.

## Independent validation of cider, wine and mead

Primary references independently reviewed on 2026-11-13; the resulting labels/prompts are Mealshop's
original implementation decisions. Validation here means source review of the process distinctions,
not certification or a guarantee that a user's recipe is safe or fermentation complete.

| Template and boundary | Source-supported distinction | Primary reference and version |
| --- | --- | --- |
| Cider from prepared apple juice | Juice preparation, yeast, observations, transfer and maturation; no grain conversion. Whole-fruit milling/pressing is outside this juice starter template. | [AHA, How to Make Cider](https://homebrewersassociation.org/tutorials/how-to-make-cider/how-to-make-cider/), 2018-02-15 |
| Traditional mead without boiling | Honey dissolved in water, yeast, recipe-specific nutrition/observations, transfer and maturation. No malt conversion or mandatory wort boil. | [AHA, How to Make Mead](https://homebrewersassociation.org/tutorials/how-to-make-mead/how-to-make-mead/), 2018-02-07 |
| White wine from grapes | Press/separate solids **before** juice fermentation; then transfer and maturation. | [MoreWine, Guide to White Winemaking](https://morewinemaking.com/web_files/intranet.morebeer.com/files/wwhiw.pdf), copyright 2009, chapters 2–4, 6–7 |
| Red wine with skin fermentation | Crush/prepare grapes, ferment on skins with recipe-specific cap management, then press, transfer and mature. | [MoreWine, Guide to Red Winemaking](https://morewinemaking.com/web_files/intranet.morebeer.com/files/wredw.pdf), copyright 2008, chapters 2–4, 7–9 |

No universal temperature, elapsed fermentation duration, yeast nutrition, acid/sulphite dose, stabilization,
malolactic treatment, priming or packaging readiness is inferred. Additional repeated observation/addition
phases and source instructions can be authored explicitly in the recipe. Readings retain original SG,
Plato/Brix or temperature units; conversion, alcohol and chemistry calculators are not introduced.
These four templates reuse only neutral scheduling, equipment reservation and provenance mechanisms.
Backend validation rejects grain-method fields masquerading as non-beer processes; beverage identity
survives batch snapshots, restart and idempotent replay. Old beer/food wire formats remain unchanged.
