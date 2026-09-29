pub mod cli;
pub mod engine;
pub mod output;
pub mod scenario;
pub mod theme;

/// Generates ecommerce data with the configured calendar, seed, and population.
///
/// # Errors
/// Returns an error when the calendar, scenario, or output cannot be generated.
pub fn run(config: &engine::RunConfig) -> anyhow::Result<std::collections::BTreeMap<String, u64>> {
    run_with_theme(config, theme::Theme::load("plain")?)
}

/// Generates ecommerce data using a validated naming pack.
///
/// # Errors
/// Returns an error when theme compatibility, generation, or output fails.
pub fn run_with_theme(
    config: &engine::RunConfig,
    theme: theme::Theme,
) -> anyhow::Result<std::collections::BTreeMap<String, u64>> {
    run_scenario(config, theme, scenario::ScenarioKind::Ecommerce)
}

/// Generates the selected business model using a compatible naming pack.
///
/// # Errors
/// Returns an error when calibration, theme validation, generation, or output fails.
pub fn run_scenario(
    config: &engine::RunConfig,
    theme: theme::Theme,
    kind: scenario::ScenarioKind,
) -> anyhow::Result<std::collections::BTreeMap<String, u64>> {
    if kind == scenario::ScenarioKind::Saas {
        return run_saas(config, theme);
    }
    theme.validate(&scenario::ecommerce::Ecommerce::theme_requirements())?;
    let mut config = config.clone();
    if let Some(target) = config.target_rows {
        if !config.quiet {
            eprintln!("Calibrating orders for --target-rows {target}...");
        }
        let max_days =
            usize::try_from(config.start_date.until(jiff::civil::Date::MAX)?.get_days())? + 1;
        let calibration = engine::calibration::calibrate(
            |days| {
                let calendar = engine::calendar::precompute(config.start_date, days)?;
                scenario::ecommerce::Ecommerce::with_theme(
                    config.seed,
                    config.scale,
                    calendar,
                    theme.clone(),
                )
            },
            config.seed,
            "orders",
            u64::try_from(target)?,
            max_days,
        )?;
        config.days = calibration.days;
    }
    let days = engine::calendar::precompute(config.start_date, config.days)?;
    let mut scenario =
        scenario::ecommerce::Ecommerce::with_theme(config.seed, config.scale, days, theme)?;
    engine::run(&mut scenario, &config)
}

fn run_saas(
    config: &engine::RunConfig,
    theme: theme::Theme,
) -> anyhow::Result<std::collections::BTreeMap<String, u64>> {
    use scenario::saas::Saas;
    theme.validate(&Saas::theme_requirements())?;
    let mut config = config.clone();
    if let Some(target) = config.target_rows {
        if !config.quiet {
            eprintln!("Calibrating accounts for --target-rows {target}...");
        }
        let max_days =
            usize::try_from(config.start_date.until(jiff::civil::Date::MAX)?.get_days())? + 1;
        config.days = engine::calibration::calibrate(
            |days| {
                Saas::with_theme(
                    config.seed,
                    config.scale,
                    config.start_date,
                    days,
                    theme.clone(),
                )
            },
            config.seed,
            "accounts",
            u64::try_from(target)?,
            max_days,
        )?
        .days;
    }
    let mut scenario = Saas::with_theme(
        config.seed,
        config.scale,
        config.start_date,
        config.days,
        theme,
    )?;
    engine::run(&mut scenario, &config)
}

#[must_use]
pub fn annotation_probe() -> i32 {
    return 1;
}
