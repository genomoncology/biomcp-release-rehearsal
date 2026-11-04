use super::{Fields, TrialSearch, TrialSource};
use crate::{NctId, QueryInvalid};

impl TryFrom<Fields> for TrialSearch {
    type Error = QueryInvalid;
    fn try_from(mut f: Fields) -> Result<Self, Self::Error> {
        if let Some(id) = f
            .query
            .as_ref()
            .and_then(|q| q.as_str().parse::<NctId>().ok())
        {
            if f.id.is_some_and(|explicit| explicit != id) {
                return Err(QueryInvalid::new(&["query", "id"]));
            }
            f.id = Some(id);
            f.query = None;
        }
        if let Some(id) = &f.condition_id {
            if !f.condition.is_empty() {
                return Err(QueryInvalid::new(&["condition_id", "condition"]));
            }
            if !f.keyword.is_empty() {
                return Err(QueryInvalid::new(&["condition_id", "keyword"]));
            }
            if !valid_condition_id(id, f.source) {
                return Err(QueryInvalid::new(&["condition_id", "source"]));
            }
        }
        validate_source_filters(&f)?;
        validate_biomarker(&f)?;
        validate_alternatives(&f)?;
        super::country::validate(&f.country)?;
        validate_exact_condition(&f)?;
        super::continuation::validate(&f)?;
        if !(1..=100).contains(&f.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if f.age.is_some_and(|v| v > 150) {
            return Err(QueryInvalid::new(&["age"]));
        }
        if f.updated_since
            .zip(f.updated_until)
            .is_some_and(|(a, b)| a.to_string() > b.to_string())
        {
            return Err(QueryInvalid::new(&["updated_since", "updated_until"]));
        }
        validate_location(&f)?;
        Ok(Self(Box::new(f)))
    }
}

fn validate_exact_condition(f: &Fields) -> Result<(), QueryInvalid> {
    if f.exact_condition && f.source != TrialSource::Ctgov {
        return Err(QueryInvalid::new(&["exact_condition", "source"]));
    }
    if f.exact_condition && f.condition.is_empty() {
        return Err(QueryInvalid::new(&["exact_condition", "condition"]));
    }
    Ok(())
}

fn valid_condition_id(id: &biodata::Curie, source: TrialSource) -> bool {
    let value = id.to_string();
    let Some((namespace, local)) = value.split_once(':') else {
        return false;
    };
    match namespace {
        "MONDO" | "DOID" => !local.is_empty() && local.bytes().all(|v| v.is_ascii_digit()),
        "NCIT" => {
            source == TrialSource::Nci
                && local
                    .strip_prefix('C')
                    .is_some_and(|v| !v.is_empty() && v.bytes().all(|v| v.is_ascii_digit()))
        }
        _ => false,
    }
}

fn validate_alternatives(f: &Fields) -> Result<(), QueryInvalid> {
    for (name, values) in [
        ("condition", &f.condition),
        ("intervention", &f.intervention),
        ("keyword", &f.keyword),
    ] {
        if values.len() > super::MAX_ALTERNATIVES {
            return Err(QueryInvalid::new(&[name]));
        }
    }
    Ok(())
}

fn validate_biomarker(f: &Fields) -> Result<(), QueryInvalid> {
    if (f.biomarker_code.is_some() || f.biomarker_use.is_some()) && f.source != TrialSource::Nci {
        return Err(QueryInvalid::new(&[
            "biomarker_code",
            "biomarker_use",
            "source",
        ]));
    }
    if f.biomarker_use.is_some() && f.biomarker_code.is_none() {
        return Err(QueryInvalid::new(&["biomarker_code", "biomarker_use"]));
    }
    Ok(())
}

fn validate_source_filters(f: &Fields) -> Result<(), QueryInvalid> {
    if f.status.as_ref().is_some_and(|v| !v.valid_for(f.source)) {
        return Err(QueryInvalid::new(&["status", "source"]));
    }
    if f.phase.as_ref().is_some_and(|v| !v.valid_for(f.source)) {
        return Err(QueryInvalid::new(&["phase", "source"]));
    }
    Ok(())
}

fn validate_location(f: &Fields) -> Result<(), QueryInvalid> {
    if f.facility.is_some() && (f.lat.is_some() || f.lon.is_some() || f.distance.is_some()) {
        return Err(QueryInvalid::new(&["facility", "lat", "lon", "distance"])
            .with_message(crate::ErrorMessage::TrialFacilityRadius));
    }
    if [f.lat.is_some(), f.lon.is_some(), f.distance.is_some()]
        .iter()
        .filter(|v| **v)
        .count()
        % 3
        != 0
    {
        return Err(QueryInvalid::new(&["lat", "lon", "distance"]));
    }
    if let (Some(lat), Some(lon), Some(distance)) = (&f.lat, &f.lon, &f.distance) {
        let lat = lat
            .as_str()
            .parse::<f64>()
            .map_err(|_| QueryInvalid::new(&["lat"]))?;
        let lon = lon
            .as_str()
            .parse::<f64>()
            .map_err(|_| QueryInvalid::new(&["lon"]))?;
        let distance = distance
            .as_str()
            .strip_suffix("mi")
            .or_else(|| distance.as_str().strip_suffix("km"))
            .ok_or_else(|| QueryInvalid::new(&["distance"]))?
            .parse::<f64>()
            .map_err(|_| QueryInvalid::new(&["distance"]))?;
        if !(-90.0..=90.0).contains(&lat) {
            return Err(QueryInvalid::new(&["lat"]));
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Err(QueryInvalid::new(&["lon"]));
        }
        if !distance.is_finite() {
            return Err(QueryInvalid::new(&["distance"]));
        }
        if distance <= 0.0 {
            return Err(QueryInvalid::new(&["distance"]));
        }
    }
    Ok(())
}
