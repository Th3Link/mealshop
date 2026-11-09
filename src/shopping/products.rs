use super::model::{Data, Item, Origin, Quantity, Requirement, Source, amount, text};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Products {
    pub(crate) records: BTreeMap<String, Product>,
    pub(crate) candidates: Vec<Candidate>,
    pub(crate) last_lookup: u64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Evidence {
    pub(crate) source_url: String,
    pub(crate) licence: String,
    pub(crate) retrieved_at: u64,
    pub(crate) modified_at: Option<u64>,
    pub(crate) original_name: String,
    pub(crate) original_brand: String,
    pub(crate) original_quantity: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Candidate {
    pub(crate) id: String,
    pub(crate) barcode: String,
    pub(crate) evidence: Evidence,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Product {
    pub(crate) barcode: String,
    pub(crate) name: String,
    pub(crate) brand: String,
    pub(crate) label_quantity: String,
    pub(crate) quantity: Quantity,
    pub(crate) ingredient_id: Option<String>,
    pub(crate) evidence: Option<Evidence>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProductInput {
    pub(crate) barcode: String,
    pub(crate) name: String,
    pub(crate) brand: String,
    pub(crate) label_quantity: String,
    pub(crate) quantity: Quantity,
    pub(crate) ingredient_id: Option<String>,
    pub(crate) match_confirmed: bool,
    pub(crate) candidate_id: Option<String>,
}
pub(crate) fn gtin(code: &str) -> Result<String, &'static str> {
    if !matches!(code.len(), 8 | 12 | 13 | 14)
        || !code.bytes().all(|b| b.is_ascii_digit())
        || code.bytes().all(|b| b == b'0')
    {
        return Err("shopping_barcode_invalid");
    }
    let sum: u32 = code
        .bytes()
        .rev()
        .enumerate()
        .map(|(i, b)| u32::from(b - b'0') * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    if sum % 10 != 0 {
        return Err("shopping_barcode_invalid");
    }
    Ok(format!("{code:0>14}"))
}
fn optional_text(value: &str, max: usize) -> bool {
    value.is_empty() || text(value, max)
}
pub(crate) fn evidence_valid(e: &Evidence, barcode: &str) -> bool {
    e.source_url == format!("https://world.openfoodfacts.org/product/{barcode}")
        && e.licence == "ODbL-1.0"
        && e.retrieved_at > 0
        && e.modified_at
            .is_none_or(|t| t > 0 && t <= e.retrieved_at.saturating_add(86400))
        && text(&e.original_name, 200)
        && optional_text(&e.original_brand, 200)
        && optional_text(&e.original_quantity, 200)
}
pub(crate) fn product_valid(p: &Product) -> bool {
    gtin(&p.barcode).is_ok()
        && text(&p.name, 200)
        && optional_text(&p.brand, 200)
        && optional_text(&p.label_quantity, 200)
        && p.quantity.normalized().is_ok()
        && p.ingredient_id.as_ref().is_none_or(|id| text(id, 120))
        && p.evidence
            .as_ref()
            .is_none_or(|e| evidence_valid(e, &p.barcode))
}
impl Data {
    pub(crate) fn save_product(
        &mut self,
        input: ProductInput,
        catalogue: &crate::nutrition::catalogue::Catalogue,
        now: u64,
    ) -> Result<(), &'static str> {
        let canonical = gtin(&input.barcode)?;
        if let Some(id) = &input.ingredient_id {
            if !input.match_confirmed {
                return Err("shopping_match_required");
            }
            if !catalogue.ingredients.iter().any(|i| &i.id == id) {
                return Err("unknown_ingredient");
            }
        }
        let evidence = if let Some(id) = input.candidate_id {
            let candidate = self
                .products
                .candidates
                .iter()
                .find(|c| {
                    c.id == id
                        && gtin(&c.barcode).ok().as_ref() == Some(&canonical)
                        && now.saturating_sub(c.evidence.retrieved_at) <= 86400
                })
                .ok_or("shopping_candidate_expired")?;
            let mut evidence = candidate.evidence.clone();
            evidence.source_url =
                format!("https://world.openfoodfacts.org/product/{}", input.barcode);
            Some(evidence)
        } else {
            self.products
                .records
                .get(&canonical)
                .and_then(|p| p.evidence.clone())
                .map(|mut e| {
                    e.source_url =
                        format!("https://world.openfoodfacts.org/product/{}", input.barcode);
                    e
                })
        };
        let product = Product {
            barcode: input.barcode,
            name: input.name,
            brand: input.brand,
            label_quantity: input.label_quantity,
            quantity: input.quantity,
            ingredient_id: input.ingredient_id,
            evidence,
        };
        if !product_valid(&product) {
            return Err("shopping_product_invalid");
        }
        self.products.records.insert(canonical, product);
        Ok(())
    }
    pub(crate) fn add_product(
        &mut self,
        barcode: &str,
        packages: u32,
        catalogue: &crate::nutrition::catalogue::Catalogue,
    ) -> Result<(), &'static str> {
        if !(1..=1_000_000).contains(&packages) {
            return Err("shopping_product_invalid");
        }
        let product = self
            .products
            .records
            .get(&gtin(barcode)?)
            .ok_or("shopping_product_not_found")?
            .clone();
        let (unit, quantity) = product.quantity.normalized()?;
        let amount = amount(quantity.atoms() * u128::from(packages))?;
        let id = self.allocate("source")?;
        let ingredient = product
            .ingredient_id
            .as_ref()
            .map(|id| {
                catalogue
                    .ingredients
                    .iter()
                    .find(|i| &i.id == id)
                    .ok_or("unknown_ingredient")
            })
            .transpose()?;
        let name = ingredient.map_or_else(|| product.name.clone(), |i| i.name.clone());
        let translations = ingredient.map_or_else(BTreeMap::new, |i| {
            i.translations
                .iter()
                .map(|(lang, tr)| (lang.clone(), tr.name.clone()))
                .collect()
        });
        let key = product.ingredient_id.as_ref().map_or_else(
            || format!("manual:{id}:{}", unit.key()),
            |ingredient| format!("ingredient:{ingredient}:{}", unit.key()),
        );
        self.sources.push(Source {
            id,
            active: true,
            origin: Origin::Product {
                product: product.clone(),
                packages,
            },
            requirements: vec![Requirement {
                item: Item {
                    key,
                    ingredient_id: product.ingredient_id,
                    name,
                    translations,
                    unit,
                },
                amount,
            }],
        });
        Ok(())
    }
    pub(crate) fn validate_products(&self) -> Result<(), &'static str> {
        if self.products.records.len() > 1000 || self.products.candidates.len() > 20 {
            return Err("shopping_capacity");
        }
        let mut ids = std::collections::BTreeSet::new();
        if self
            .products
            .records
            .iter()
            .any(|(key, p)| gtin(&p.barcode).as_ref() != Ok(key) || !product_valid(p))
            || self.products.candidates.iter().any(|c| {
                !ids.insert(&c.id)
                    || !self.allocated_id(&c.id, "lookup-")
                    || gtin(&c.barcode).is_err()
                    || !evidence_valid(&c.evidence, &c.barcode)
            })
        {
            return Err("shopping_data_invalid");
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn only_retained_unexpired_candidates_can_supply_provider_attribution() {
        let catalogue: crate::nutrition::catalogue::Catalogue =
            serde_json::from_str(include_str!("../../data/catalogue.json")).unwrap();
        let mut data = Data::default();
        let id = data.allocate("lookup").unwrap();
        let evidence = Evidence {
            source_url: "https://world.openfoodfacts.org/product/034000470693".to_owned(),
            licence: "ODbL-1.0".to_owned(),
            retrieved_at: 100,
            modified_at: Some(50),
            original_name: "Original label".to_owned(),
            original_brand: "Original brand".to_owned(),
            original_quantity: "2 x 50 g".to_owned(),
        };
        data.products.candidates.push(Candidate {
            id: id.clone(),
            barcode: "034000470693".to_owned(),
            evidence,
        });
        let input = |barcode: &str| ProductInput {
            barcode: barcode.to_owned(),
            name: "Reviewed label".to_owned(),
            brand: String::new(),
            label_quantity: "2 x 50 g".to_owned(),
            quantity: Quantity {
                amount: amount(100_000_000).unwrap(),
                unit: super::super::model::Unit::G,
            },
            ingredient_id: None,
            match_confirmed: false,
            candidate_id: Some(id.clone()),
        };
        assert!(
            data.save_product(input("12345670"), &catalogue, 200)
                .is_err()
        );
        assert!(
            data.save_product(input("034000470693"), &catalogue, 90_000)
                .is_err()
        );
        assert!(data.products.records.is_empty());
        data.save_product(input("0034000470693"), &catalogue, 200)
            .unwrap();
        let product = &data.products.records["00034000470693"];
        assert_eq!(product.name, "Reviewed label");
        assert_eq!(
            product.evidence.as_ref().unwrap().original_name,
            "Original label"
        );
        assert_eq!(
            product.evidence.as_ref().unwrap().source_url,
            "https://world.openfoodfacts.org/product/0034000470693"
        );
        assert_eq!(product.ingredient_id, None);
        assert!(data.validate_products().is_ok());
    }
}
