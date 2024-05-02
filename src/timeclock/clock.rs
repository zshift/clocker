use core::time;

use anyhow::Result;
use chrono::{Local, TimeDelta};
use cli_table::{format::Justify, print_stdout, Cell, Color, Style, Table};

use crate::Granularity;

use super::timesheet::*;
/// Debug mode.
pub enum Debug {
    On,
    Off,
}

impl Debug {
    pub fn is_on(&self) -> bool {
        match self {
            Debug::On => true,
            Debug::Off => false,
        }
    }
}

fn print_hms(time: TimeDelta) {
    let hh = time.num_hours();
    let mm = time.num_minutes() % 60;
    let ss = time.num_seconds() % 60;

    println!("{:02}:{:02}:{:02}", hh, mm, ss);
}

/// Timeclock service
pub struct Timeclock<'a> {
    timesheet_path: &'a std::path::Path,
    debug: Debug,
}

impl<'a> Timeclock<'a> {
    /// Creates a new Clock instance.
    pub fn new(timesheet_path: &'a std::path::Path, debug: Debug) -> Self {
        Self {
            timesheet_path,
            debug,
        }
    }

    /// Clocks in the user.
    pub fn clock_in(&self) -> Result<()> {
        let mut timesheet = self.get_timesheet()?;

        if let Some(Action::In(_)) = timesheet.last_action() {
            anyhow::bail!("You are already clocked in");
        }

        timesheet.clock_in(Local::now());
        self.save_timesheet(&timesheet)?;

        Ok(())
    }

    /// Clocks out the user.
    pub fn clock_out(&self) -> Result<()> {
        let mut timesheet = self.get_timesheet()?;

        if let Some(Action::Out(_)) = timesheet.last_action() {
            anyhow::bail!("You are already clocked out");
        }

        timesheet.clock_out(Local::now());
        self.save_timesheet(&timesheet)?;

        Ok(())
    }

    /// Prints the total time worked.
    pub fn time_clocked(&self, worked: &This) -> Result<()> {
        let timesheet = self.get_timesheet()?;
        let total_time = timesheet.total_time(worked);

        print_hms(total_time);

        Ok(())
    }

    /// Returns the total time worked today.
    pub fn running_time(&self) -> Result<()> {
        let timesheet = self.get_timesheet()?;
        let running_time = timesheet.running_time().unwrap_or(TimeDelta::zero());

        print_hms(running_time);

        Ok(())
    }

    /// Prints the raw timesheet.
    pub fn raw_timesheet(&self) -> Result<()> {
        let timesheet = &self.get_timesheet()?;
        let timesheet = serde_json::to_string(&timesheet)?;

        println!("{}", timesheet);
        Ok(())
    }

    pub fn timesheet(&self) -> Result<()> {
        let timesheet = &self.get_timesheet()?;
        let weekly_hours = timesheet
            .weekly_hours()
            .iter()
            .map(|hours| hours.cell())
            .collect::<Vec<_>>();
        let chart = vec![weekly_hours]
            .table()
            .title(vec![
                "Monday"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(Color::Green)),
                "Tuesday"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(Color::Green)),
                "Wednesday"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(Color::Green)),
                "Thursday"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(Color::Green)),
                "Friday"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(Color::Green)),
                "Saturday"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(Color::Yellow)),
                "Sunday"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(Color::Yellow)),
            ])
            .bold(true);

        print_stdout(chart)?;
        Ok(())
    }

    pub fn watch(&self, _hours: &usize) {}

    /// Wipes the timesheet.
    pub fn wipe(&self) -> Result<()> {
        //let timesheet = Timesheet::default();
        //self.save_timesheet(&timesheet)?;

        Ok(())
    }

    fn get_timesheet(&self) -> Result<Timesheet> {
        if self.debug.is_on() {
            eprintln!("Loading timesheet from: {:?}", self.timesheet_path);
        }

        if self.timesheet_path.exists() {
            let timesheet = std::fs::read_to_string(self.timesheet_path)?;
            let timesheet = serde_json::from_str(&timesheet)?;
            Ok(timesheet)
        } else {
            eprintln!("No timesheet found, creating a new one.");
            Ok(Timesheet::default())
        }
    }

    fn save_timesheet(&self, timesheet: &Timesheet) -> Result<()> {
        if self.debug.is_on() {
            eprintln!("Saving timesheet to: {:?}", self.timesheet_path);
        }

        let timesheet = serde_json::to_string_pretty(timesheet)?;
        std::fs::write(self.timesheet_path, timesheet)?;

        Ok(())
    }
}

#[cfg(test)]
mod timeclock_tests {
    use super::*;
    use chrono::{DurationRound, Timelike};
    use tempfile::tempdir;

    fn round(time: DateTime) -> DateTime {
        time.duration_round(TimeDelta::try_seconds(1).unwrap())
            .unwrap()
    }

    fn with_temp(f: impl FnOnce(&std::path::Path) -> Result<()>) -> Result<()> {
        let temp_dir = tempdir()?;
        let timesheet_path = temp_dir.path().join("timesheet.json");
        let ret = f(&timesheet_path);

        temp_dir.close()?;

        ret
    }

    #[test]
    fn clock_in() -> Result<()> {
        with_temp(|timesheet_path| {
            let timeclock = Timeclock::new(timesheet_path, Debug::Off);
            timeclock.clock_in()?;
            let timesheet = timeclock.get_timesheet()?;

            match timesheet.last_action().unwrap() {
                Action::In(time) => assert_eq!(round(*time), round(Local::now())),
                _ => panic!("Expected last action to be a clock in"),
            }

            Ok(())
        })
    }

    #[test]
    fn clock_in_twice_fails() -> Result<()> {
        with_temp(|timesheet_path| {
            let timeclock = Timeclock::new(timesheet_path, Debug::Off);
            timeclock.clock_in()?;
            let result = timeclock.clock_in();

            assert!(result.is_err());

            Ok(())
        })
    }

    #[test]
    fn clock_out() -> Result<()> {
        with_temp(|timesheet_path| {
            let timeclock = Timeclock::new(timesheet_path, Debug::Off);
            timeclock.clock_in()?;
            timeclock.clock_out()?;
            let timesheet = timeclock.get_timesheet()?;

            match timesheet.last_action().unwrap() {
                Action::Out(time) => assert_eq!(round(*time), round(Local::now())),
                _ => panic!("Expected last action to be a clock out"),
            }

            Ok(())
        })
    }

    #[test]
    fn clock_out_twice_fails() -> Result<()> {
        with_temp(|timesheet_path| {
            let timeclock = Timeclock::new(timesheet_path, Debug::Off);
            timeclock.clock_out()?;
            let result = timeclock.clock_out();

            assert!(result.is_err());

            Ok(())
        })
    }

    #[test]
    fn time_worked_today() -> Result<()> {
        with_temp(|timesheet_path| {
            let timeclock = Timeclock::new(timesheet_path, Debug::Off);

            // Setup
            {
                let mut timesheet = timeclock.get_timesheet()?;
                let now = Local::now().with_hour(12).unwrap();
                let clock_in = now
                    .checked_sub_signed(TimeDelta::try_hours(8).unwrap())
                    .unwrap();

                timesheet.clock_in(clock_in);
                timesheet.clock_out(now);
                timeclock.save_timesheet(&timesheet)?;
            }

            // No assertions, just make sure it doesn't panic.
            timeclock.time_clocked(&This::Day)?;

            Ok(())
        })
    }

    // #[test(skip = "Not implemented")]
    // fn wipe() -> Result<()> {
    //     with_temp(|timesheet_path| {
    //         let timeclock = Timeclock::new(timesheet_path, Debug::Off);
    //         timeclock.clock_in()?;
    //         timeclock.wipe()?;
    //         let timesheet = timeclock.get_timesheet()?;

    //         assert!(timesheet.clocks.is_empty());

    //         Ok(())
    //     })
    // }
}
