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
                scenario::ecommerce::Ecommerce::new(config.seed, config.scale, calendar)
            },
            config.seed,
            "orders",
            u64::try_from(target)?,
            max_days,
        )?;
        config.days = calibration.days;
    }
    let days = engine::calendar::precompute(config.start_date, config.days)?;
    let mut scenario = scenario::ecommerce::Ecommerce::new(config.seed, config.scale, days)?;
    engine::run(&mut scenario, &config)
}
