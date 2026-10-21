use chrono::{Datelike, NaiveDate};

use super::Translator;

/// Writes dates the way people say them in each language.
pub struct DateFormatter<'translator> {
    translator: &'translator Translator,
}

impl<'translator> DateFormatter<'translator> {
    pub fn new(translator: &'translator Translator) -> Self {
        Self { translator }
    }

    /// "miércoles 21 de octubre" / "Wednesday, October 21".
    pub fn long(&self, date: NaiveDate) -> String {
        let names = &self.translator.catalog().dates;
        self.translator.format(
            "date.long",
            &[
                ("weekday", &names.weekdays[Self::weekday_index(date)]),
                ("day", &date.day().to_string()),
                ("month", &names.months[Self::month_index(date)]),
            ],
        )
    }

    /// "Mié 21" / "Wed 21".
    pub fn short(&self, date: NaiveDate) -> String {
        let names = &self.translator.catalog().dates;
        self.translator.format(
            "date.short",
            &[
                (
                    "weekday_short",
                    &names.weekdays_short[Self::weekday_index(date)],
                ),
                ("day", &date.day().to_string()),
            ],
        )
    }

    fn weekday_index(date: NaiveDate) -> usize {
        date.weekday().num_days_from_monday() as usize
    }

    fn month_index(date: NaiveDate) -> usize {
        date.month0() as usize
    }
}
