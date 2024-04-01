use std::collections::VecDeque;

use chrono::{Datelike, Days, Local, NaiveDate, TimeDelta};
use serde::{Deserialize, Serialize};

pub type DateTime = chrono::DateTime<Local>;

/// Represents a clock in or out action.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Action {
    In(DateTime),
    Out(DateTime),
}

#[derive(Debug)]
pub enum This {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Timesheet {
    pub clocks: VecDeque<Action>,
}

impl Timesheet {
    /// Returns the total time worked in hours.
    pub fn total_time(&self, worked: &This) -> TimeDelta {
        let mut total_time = TimeDelta::zero();

        let today = Local::now();
        let mut last_clock_in = None;
        let clocks: Vec<&Action> = match worked {
            This::Day => self
                .clocks
                .iter()
                .filter(|action| match action {
                    Action::In(time) => time.date_naive() == today.date_naive(),
                    Action::Out(time) => time.date_naive() == today.date_naive(),
                })
                .collect(),
            This::Week => {
                fn closest_prev_monday(date: NaiveDate) -> NaiveDate {
                    let days_so_far = date.weekday().num_days_from_monday();
                    date.checked_sub_days(Days::new(days_so_far as u64))
                        .unwrap()
                }

                let monday = closest_prev_monday(today.date_naive());
                self.clocks
                    .iter()
                    .filter(|action| match action {
                        Action::In(time) => time.date_naive() >= monday,
                        Action::Out(time) => time.date_naive() >= monday,
                    })
                    .collect()
            }
            This::Month => {
                let start_of_month = today.date_naive().with_day(1).unwrap();
                self.clocks
                    .iter()
                    .filter(|action| match action {
                        Action::In(time) => time.date_naive() >= start_of_month,
                        Action::Out(time) => time.date_naive() >= start_of_month,
                    })
                    .collect()
            }
            This::Year => {
                let start_of_year = today
                    .date_naive()
                    .with_month(1)
                    .unwrap()
                    .with_day(1)
                    .unwrap();
                self.clocks
                    .iter()
                    .filter(|action| match action {
                        Action::In(time) => time.date_naive() >= start_of_year,
                        Action::Out(time) => time.date_naive() >= start_of_year,
                    })
                    .collect()
            }
        };

        for action in clocks {
            match action {
                Action::In(time) => {
                    last_clock_in = Some(time);
                }
                Action::Out(time) => {
                    if let Some(last_clock_in) = last_clock_in {
                        total_time += time.signed_duration_since(*last_clock_in);
                    }
                }
            }
        }

        total_time
    }

    // Clocks in.
    pub fn clock_in(&mut self, when: DateTime) {
        self.clocks.push_back(Action::In(when));
    }

    // Clocks out.
    pub fn clock_out(&mut self, when: DateTime) {
        self.clocks.push_back(Action::Out(when));
    }

    /// Returns the last action in the timesheet.
    pub fn last_action(&self) -> Option<&Action> {
        self.clocks.back()
    }

    /// Returns the amount of time worked since clocking in today.
    /// TODO: This is a bit of a mess, needs to be refactored.
    pub fn running_time(&self) -> Option<TimeDelta> {
        let now = Local::now();
        let today = now.date_naive();

        let clocks = self
            .clocks
            .iter()
            .filter(|&clock| match clock {
                Action::In(time) => time.date_naive() == today,
                Action::Out(time) => time.date_naive() == today,
            })
            .collect::<Vec<_>>();

        let mut last_clock_in = None;
        let mut total_time = TimeDelta::zero();

        for action in clocks {
            match action {
                Action::In(time) => {
                    last_clock_in = Some(time);
                }
                Action::Out(time) => {
                    if let Some(clock) = last_clock_in {
                        total_time += time.signed_duration_since(*clock);
                        last_clock_in = None;
                    }
                }
            }
        }

        if let Some(last_clock_in) = last_clock_in {
            Some(total_time + now.signed_duration_since(*last_clock_in))
        } else {
            Some(total_time)
        }
    }
}

#[cfg(test)]
mod timesheet_tests {
    use super::*;

    #[test]
    fn total_time_today() {
        let mut timesheet = Timesheet::default();
        let now = Local::now();
        let clock_in = now
            .checked_sub_signed(TimeDelta::try_hours(8).unwrap())
            .unwrap();
        let clock_out = now;

        timesheet.clocks.push_back(Action::In(clock_in));
        timesheet.clocks.push_back(Action::Out(clock_out));

        let total_time = timesheet.total_time(&This::Day);
        assert_eq!(total_time.num_hours(), 8);
    }

    #[test]
    fn total_time_this_week() {
        let mut timesheet = Timesheet::default();
        let now = Local::now();
        let clock_in = now
            .checked_sub_signed(TimeDelta::try_hours(8).unwrap())
            .unwrap();
        let clock_out = now;

        timesheet.clock_in(clock_in);
        timesheet.clock_out(clock_out);

        let total_time = timesheet.total_time(&This::Week);
        assert_eq!(total_time.num_hours(), 8);
    }

    #[test]
    fn total_time_this_month() {
        let mut timesheet = Timesheet::default();
        let now = Local::now();
        let clock_in = now
            .checked_sub_signed(TimeDelta::try_hours(8).unwrap())
            .unwrap();
        let clock_out = now;

        timesheet.clock_in(clock_in);
        timesheet.clock_out(clock_out);

        let total_time = timesheet.total_time(&This::Month);
        assert_eq!(total_time.num_hours(), 8);
    }

    #[test]
    fn total_time_this_year() {
        let mut timesheet = Timesheet::default();
        let now = Local::now();
        let clock_in = now
            .checked_sub_signed(TimeDelta::try_hours(8).unwrap())
            .unwrap();
        let clock_out = now;

        timesheet.clock_in(clock_in);
        timesheet.clock_out(clock_out);

        let total_time = timesheet.total_time(&This::Year);
        assert_eq!(total_time.num_hours(), 8);
    }

    #[test]
    fn last_action() {
        let mut timesheet = Timesheet::default();
        let now = Local::now();
        let clock_in = now
            .checked_sub_signed(TimeDelta::try_hours(8).unwrap())
            .unwrap();
        let clock_out = now;

        timesheet.clock_in(clock_in);
        timesheet.clock_out(clock_out);

        assert_eq!(*timesheet.last_action().unwrap(), Action::Out(clock_out));
    }

    #[test]
    fn running_time() {
        let mut timesheet = Timesheet::default();
        let now = Local::now();
        let clock_in = now
            .checked_sub_signed(TimeDelta::try_hours(8).unwrap())
            .unwrap();

        timesheet.clock_in(clock_in);

        let running_time = timesheet.running_time().unwrap();
        assert_eq!(
            running_time
                .checked_sub(&TimeDelta::nanoseconds(running_time.subsec_nanos() as i64))
                .unwrap(),
            TimeDelta::try_hours(8).unwrap()
        )
    }
}
