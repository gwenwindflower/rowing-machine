use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use jiff::{Span, civil::Date};

use crate::{engine::RunConfig, output::Format};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Generate deterministic business data for SQL training and analytics demos"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    /// Naming pack or TOML path; `--theme fantasy_rpg` uses Arcanum Collective vocabulary
    #[arg(long, default_value = "plain")]
    pub theme: String,
    /// Business model to simulate
    #[arg(long, value_enum, default_value = "ecommerce")]
    pub scenario: crate::scenario::ScenarioKind,
    /// Number of 365-day years to simulate
    #[arg(long, default_value = "4", value_parser = positive, allow_hyphen_values = true)]
    pub years: usize,
    /// Choose duration for about this many orders (ecommerce) or accounts (saas); --target-rows 1000 (default: unset)
    #[arg(long, value_parser = positive, allow_hyphen_values = true, conflicts_with = "years")]
    pub target_rows: Option<usize>,
    /// Population multiplier; --scale 10 gives each store ten times its base population or 200 addressable software accounts
    #[arg(long, default_value = "100", value_parser = positive, allow_hyphen_values = true)]
    pub scale: usize,
    /// Reproducible random seed; 0 chooses and prints a random seed
    #[arg(long, default_value = "0", value_parser = seed, allow_hyphen_values = true)]
    pub seed: u64,
    /// First simulation day in YYYY-MM-DD format
    #[arg(long, default_value = "2023-01-01", value_parser = date)]
    pub start_date: Date,
    /// Output file format
    #[arg(long, value_enum, default_value = "csv")]
    pub format: Format,
    /// Compress JSONL with gzip or Parquet with zstd; --format jsonl --compress (default: false)
    #[arg(long)]
    pub compress: bool,
    /// Directory for generated files, created if missing
    #[arg(long, default_value = "./factory-output")]
    pub output_dir: PathBuf,
    /// Filename prefix; --pre raw writes `raw_orders.csv`, `raw_customers.csv`, etc.
    #[arg(long, default_value = "raw", value_parser = prefix)]
    pub pre: String,
    /// Suppress progress, seed, and row summary (default: false)
    #[arg(long)]
    pub quiet: bool,
    /// Worker threads; 1 runs serially (default: available cores)
    #[arg(long, default_value_t = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get), value_parser = positive, allow_hyphen_values = true)]
    pub workers: usize,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List bundled naming packs and their compatible scenarios
    Themes,
}

impl Cli {
    /// Lists bundled themes or generates data from validated flags.
    ///
    /// # Errors
    /// Returns actionable errors for invalid flags, themes, or output failures.
    pub fn run(self) -> Result<()> {
        use crate::{scenario::ScenarioKind, theme::Theme};

        if matches!(self.command, Some(Command::Themes)) {
            for theme in Theme::bundled()? {
                let scenarios = ScenarioKind::ALL
                    .into_iter()
                    .filter(|kind| theme.is_compatible(&kind.theme_requirements()))
                    .map(ScenarioKind::name)
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("{}\t{}\t{}", theme.name, theme.description, scenarios);
            }
            return Ok(());
        }
        let theme = Theme::load(&self.theme)?;
        theme
            .validate(&self.scenario.theme_requirements())
            .with_context(|| {
                format!(
                    "--theme {:?} is incompatible with {}",
                    self.theme,
                    self.scenario.name()
                )
            })?;
        let scenario = self.scenario;
        crate::run_scenario(&self.config()?, theme, scenario)?;
        Ok(())
    }

    /// Validates run bounds and resolves a random seed when requested.
    ///
    /// # Errors
    /// Returns an actionable flag error when population or calendar bounds overflow.
    pub fn config(self) -> Result<RunConfig> {
        ensure!(
            !self.compress || self.format != Format::Csv,
            "--compress cannot be used with --format csv; choose --format jsonl or --format parquet"
        );
        let invalid_range = || {
            format!(
                "--years {} with --start-date {} exceeds the calendar; use fewer years or an earlier date",
                self.years, self.start_date
            )
        };
        let days = self.years.checked_mul(365).context(invalid_range())?;
        if self.target_rows.is_none() {
            let last_day = i64::try_from(days.saturating_sub(1)).context(invalid_range())?;
            let duration = Span::new().try_days(last_day).context(invalid_range())?;
            self.start_date
                .checked_add(duration)
                .context(invalid_range())?;
        }
        self.scale.checked_mul(62).with_context(|| {
            format!(
                "--scale {} exceeds the population limit; use a smaller positive integer",
                self.scale
            )
        })?;
        let mut chosen_seed = self.seed;
        while chosen_seed == 0 {
            chosen_seed = rand::random();
        }
        Ok(RunConfig {
            days,
            scale: self.scale,
            seed: chosen_seed,
            start_date: self.start_date,
            output_dir: self.output_dir,
            prefix: self.pre,
            quiet: self.quiet,
            format: self.format,
            compress: self.compress,
            target_rows: self.target_rows,
            workers: self.workers,
        })
    }
}

fn positive(value: &str) -> Result<usize, String> {
    value
        .parse()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| "use a positive integer greater than zero".to_owned())
}

fn seed(value: &str) -> Result<u64, String> {
    value
        .parse()
        .map_err(|_| "use an unsigned 64-bit integer; 0 chooses a random seed".to_owned())
}

fn date(value: &str) -> Result<Date, String> {
    if value.len() != 10 || value.as_bytes()[4] != b'-' || value.as_bytes()[7] != b'-' {
        return Err("use a valid date in YYYY-MM-DD format, for example 2023-01-01".to_owned());
    }
    value
        .parse()
        .map_err(|_| "use a valid date in YYYY-MM-DD format, for example 2023-01-01".to_owned())
}

fn prefix(value: &str) -> Result<String, String> {
    if value.is_empty() || value.contains(['/', '\\', '\0']) || matches!(value, "." | "..") {
        return Err(
            "use a non-empty filename prefix without path separators, for example raw".to_owned(),
        );
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn defaults_define_four_years_and_one_hundred_times_population() {
        let cli = Cli::parse_from(["rowing-machine"]);
        assert_eq!((cli.years, cli.scale, cli.seed), (4, 100, 0));
        assert_eq!(cli.start_date.to_string(), "2023-01-01");
        assert_eq!(cli.output_dir.to_str(), Some("./factory-output"));
        assert_eq!(cli.pre, "raw");
        assert_eq!(cli.theme, "plain");
        assert!(!cli.quiet);
        assert_eq!(
            cli.workers,
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
        );
    }
}
