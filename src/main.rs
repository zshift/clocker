use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use timeclock::{Debug, Timeclock};

mod timeclock;

const TIMESHEET_PATH: &str = "timesheet.json";

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Clock in")]
    In {
        #[arg(short, long, default_value = None)]
        at: Option<chrono::NaiveDateTime>,
    },
    #[clap(about = "Clock out")]
    Out {
        #[arg(short, long, default_value = None)]
        at: Option<chrono::NaiveDateTime>,
    },
    #[clap(about = "Get the raw timesheet")]
    Raw,
    #[clap(about = "Get the time worked today, even if you haven't clocked out yet.")]
    RunningTime,
    #[clap(about = "Get total time clocked (ins and outs paired).")]
    TimeClocked {
        #[clap(subcommand)]
        granularity: Granularity,
    },
    #[clap(about = "Prints out the timesheet as a table")]
    Timesheet {
        #[arg(short, long, default_value = None)]
        on: Option<chrono::NaiveDate>,
    },
    #[clap(about = "Watches for the specified number of hours worked this week")]
    Watch {
        #[arg(short, long)]
        hours: usize,
    },
    #[clap(about = "Returns the path to the timesheet file")]
    File,
}

/// Time clocked _this_ period.
#[derive(Clone, Copy, Debug, Subcommand)]
pub enum Granularity {
    #[clap(about = "Time clocked today")]
    Today,
    #[clap(about = "Time clocked this week")]
    Week,
    #[clap(about = "Time clocked this month")]
    Month,
    #[clap(about = "Time clocked this year")]
    Year,
}

impl From<&Granularity> for timeclock::This {
    fn from(when: &Granularity) -> Self {
        match when {
            Granularity::Today => timeclock::This::Day,
            Granularity::Week => timeclock::This::Week,
            Granularity::Month => timeclock::This::Month,
            Granularity::Year => timeclock::This::Year,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let timesheet_path = dirs::data_dir().map(|data_dir| data_dir.join(TIMESHEET_PATH));
    if timesheet_path.is_none() {
        bail!("Unable to locate timesheet.");
    }

    let timesheet_path = timesheet_path.unwrap();
    let clock = Timeclock::new(
        &timesheet_path,
        if cli.debug { Debug::On } else { Debug::Off },
    );

    match &cli.command {
        Commands::In { at } => clock.clock_in(*at)?,
        Commands::Out { at } => clock.clock_out(*at)?,
        Commands::TimeClocked { granularity } => clock.time_clocked(&granularity.into())?,
        Commands::Raw => clock.raw_timesheet()?,
        Commands::RunningTime => clock.running_time()?,
        Commands::Timesheet { on } => clock.timesheet(*on)?,
        Commands::Watch { hours } => clock.watch(hours),
        Commands::File => clock.print_file(),
    }

    Ok(())
}
