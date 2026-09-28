pub mod cli;
pub mod engine;
pub mod output;
pub mod scenario;

/// Generates ecommerce data with the configured calendar, seed, and population.
///
/// # Errors
/// Returns an error when the calendar, scenario, or output cannot be generated.
pub fn run(config: &engine::RunConfig) -> anyhow::Result<std::collections::BTreeMap<String, u64>> {
    let days = engine::calendar::precompute(config.start_date, config.days)?;
    let mut scenario = scenario::ecommerce::Ecommerce::new(config.seed, config.scale, days)?;
    engine::run(&mut scenario, config)
}
