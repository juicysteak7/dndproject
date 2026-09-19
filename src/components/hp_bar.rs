use leptos::prelude::*;

/// A hit-point bar that shifts colour as the pool drops.
#[component]
pub fn HpBar(
    current: i64,
    max: i64,
    #[prop(default = 0)] temp: i64,
) -> impl IntoView {
    let fraction = if max <= 0 {
        0.0
    } else {
        (current as f64 / max as f64).clamp(0.0, 1.0)
    };

    let severity = if current <= 0 {
        "down"
    } else if fraction <= 0.25 {
        "critical"
    } else if fraction <= 0.5 {
        "bloodied"
    } else {
        "healthy"
    };

    let width = format!("{:.1}%", fraction * 100.0);

    view! {
        <div class="hp">
            <div class="hp-track">
                <div class=format!("hp-fill {severity}") style=format!("width:{width}")></div>
            </div>
            <span class="hp-label">
                {current} "/" {max}
                {(temp > 0).then(|| format!(" (+{temp} temp)"))}
            </span>
        </div>
    }
}
