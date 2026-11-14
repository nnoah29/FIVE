//! Modèles de noms et de chemins : variables `{name}`, `{stem}`, `{modified:AAAA-MM-JJ}`, `{counter:3}`…

use super::context::FileContext;
use super::time_calc::{civil, format_date};

/// Variables acceptées (sans format) ; `modified`, `created`, `today` et `counter` acceptent un format après `:`.
pub const VARIABLES: [&str; 18] = [
    "name", "stem", "ext", "kind", "size", "parent", "counter", "modified", "created", "today", "year", "month", "day", "weekday", "hash", "width", "height",
    "duration",
];

/// Noms des variables utilisées par le modèle ; une accolade jamais fermée est une erreur.
pub fn template_variables(template: &str) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let close = rest[open..].find('}').ok_or_else(|| "accolade « { » jamais fermée".to_string())?;
        found.push(rest[open + 1..open + close].split(':').next().unwrap_or("").to_string());
        rest = &rest[open + close + 1..];
    }
    if rest.contains('}') {
        return Err("accolade « } » sans ouverture".into());
    }
    Ok(found)
}

/// Valeur d'une variable pour ce fichier ; `None` si elle est inconnue ou indisponible (une date absente, par exemple).
fn variable_value(ctx: &FileContext, name: &str, format: Option<&str>) -> Option<String> {
    let date = |at: Option<f64>, default: &str| at.map(|t| format_date(t, format.unwrap_or(default)));
    let (year, month, day) = civil(ctx.now);
    Some(match name {
        "name" => ctx.name.clone(),
        "stem" => ctx.stem(),
        "ext" => ctx.extension.clone(),
        "kind" => ctx.kind.clone(),
        "size" => ctx.size.to_string(),
        "parent" => ctx.parent_name.clone(),
        "counter" => format!("{:0width$}", ctx.counter, width = format.and_then(|f| f.parse().ok()).unwrap_or(1)),
        "modified" => date(ctx.modified, "AAAA-MM-JJ")?,
        "created" => date(ctx.created, "AAAA-MM-JJ")?,
        "today" => format_date(ctx.now, format.unwrap_or("AAAA-MM-JJ")),
        "year" => format!("{year:04}"),
        "month" => format!("{month:02}"),
        "day" => format!("{day:02}"),
        "weekday" => format!("{:?}", super::time_calc::weekday(ctx.now)).to_lowercase(),
        "hash" => ctx.hash.as_deref()?.chars().take(format.and_then(|f| f.parse().ok()).unwrap_or(8)).collect(),
        "width" => ctx.image.as_ref()?.width.to_string(),
        "height" => ctx.image.as_ref()?.height.to_string(),
        "duration" => format!("{:.0}", ctx.media.as_ref()?.duration),
        _ => return None,
    })
}

/// Remplit le modèle ; une variable inconnue ou indisponible est une erreur, jamais du texte littéral.
pub fn expand(template: &str, ctx: &FileContext) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = rest[open..].find('}').ok_or_else(|| "accolade « { » jamais fermée".to_string())?;
        let body = &rest[open + 1..open + close];
        let (name, format) = body.split_once(':').map_or((body, None), |(n, f)| (n, Some(f)));
        out.push_str(&variable_value(ctx, name, format).ok_or_else(|| format!("variable « {name} » inconnue ou indisponible pour ce fichier"))?);
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}
