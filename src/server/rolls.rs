use leptos::prelude::*;

use crate::dice::RollOutcome;

/// Roll dice notation server-side (keeps an RNG out of the wasm bundle).
#[server(RollDice, "/api")]
pub async fn roll_dice(expression: String) -> Result<RollOutcome, ServerFnError> {
    let expr = crate::dice::parse(&expression).map_err(super::to_server_err)?;
    Ok(crate::dice::roll(expr))
}
