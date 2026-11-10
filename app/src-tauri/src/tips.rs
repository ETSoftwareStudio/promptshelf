//! The one-time card that asks for a star on GitHub. It counts only what
//! happens on this computer, in `desktop-tips.json` in the state folder:
//! never in a library, never in git, never sent anywhere. The app can not
//! know if the person starred the repo, so it does not try.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use promptshelf_core::{config, fsutil};
use serde::{Deserialize, Serialize};

const DAY: u64 = 86_400;
/// The card waits this long after the first start.
const SETTLE_DAYS: u64 = 3;
/// Not Now asks once more, this many days later.
const LATER_DAYS: u64 = 30;
const CREATED: u32 = 3;
const FILL_DAYS: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Answer {
    Star,
    NotNow,
    Never,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Tips {
    /// Unix seconds of the first start that had this card.
    pub first_run: u64,
    /// New prompts made in this app.
    pub created: u32,
    /// Days (UTC) with at least one Copy or Paste of a filled prompt.
    pub fill_days: u32,
    /// The last of those days, in days since 1970.
    pub last_fill_day: u64,
    /// Not Now was clicked once; the card asks again after this time.
    pub ask_after: Option<u64>,
    /// Star or Don't Ask Again was clicked, or Not Now for the second time.
    pub done: bool,
}

impl Tips {
    fn path() -> PathBuf {
        config::state_dir().join("desktop-tips.json")
    }

    /// A missing or broken file starts again from nothing.
    pub fn load() -> Self {
        std::fs::read(Self::path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn save(&self) {
        if let Ok(bytes) = serde_json::to_vec_pretty(self) {
            let _ = fsutil::write_atomic(&Self::path(), &bytes);
        }
    }

    /// Loads the counts, changes them and saves them. A failure only loses a
    /// count; it never blocks the work that counted.
    pub fn update(change: impl FnOnce(&mut Tips, u64)) {
        let mut tips = Self::load();
        let before = tips.clone();
        change(&mut tips, now());
        if tips != before {
            tips.save();
        }
    }

    pub fn started(&mut self, now: u64) {
        if self.first_run == 0 {
            self.first_run = now;
        }
    }

    pub fn created_prompt(&mut self, now: u64) {
        self.started(now);
        self.created = self.created.saturating_add(1);
    }

    pub fn filled_prompt(&mut self, now: u64) {
        self.started(now);
        let day = now / DAY;
        if day != self.last_fill_day {
            self.last_fill_day = day;
            self.fill_days = self.fill_days.saturating_add(1);
        }
    }

    /// True when the card shows: enough use, three days after the first
    /// start, and no answer that says stop.
    pub fn should_ask(&self, now: u64) -> bool {
        let used = self.created >= CREATED || self.fill_days >= FILL_DAYS;
        let settled = self.first_run > 0 && now >= self.first_run + SETTLE_DAYS * DAY;
        let due = self.ask_after.is_none_or(|t| now >= t);
        !self.done && used && settled && due
    }

    pub fn answer(&mut self, answer: Answer, now: u64) {
        match answer {
            Answer::NotNow if self.ask_after.is_none() => {
                self.ask_after = Some(now + LATER_DAYS * DAY);
            }
            _ => self.done = true,
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: u64 = 1_800_000_000;

    fn used(created: u32) -> Tips {
        let mut t = Tips::default();
        t.started(START);
        for _ in 0..created {
            t.created_prompt(START);
        }
        t
    }

    #[test]
    fn waits_for_three_prompts_and_three_days() {
        assert!(!used(2).should_ask(START + 10 * DAY));
        assert!(!used(3).should_ask(START + 3 * DAY - 1));
        assert!(used(3).should_ask(START + 3 * DAY));
    }

    #[test]
    fn five_days_with_fills_count_and_one_day_counts_once() {
        let mut t = Tips::default();
        for day in 0..4 {
            t.filled_prompt(START + day * DAY);
            t.filled_prompt(START + day * DAY + 60);
        }
        assert_eq!(t.fill_days, 4);
        assert!(!t.should_ask(START + 10 * DAY));
        t.filled_prompt(START + 9 * DAY);
        assert!(t.should_ask(START + 10 * DAY));
    }

    #[test]
    fn not_now_asks_once_more_after_thirty_days() {
        let mut t = used(3);
        let at = START + 4 * DAY;
        t.answer(Answer::NotNow, at);
        assert!(!t.should_ask(at + 29 * DAY));
        assert!(t.should_ask(at + 30 * DAY));
        t.answer(Answer::NotNow, at + 30 * DAY);
        assert!(!t.should_ask(at + 400 * DAY));
    }

    #[test]
    fn star_and_never_end_it() {
        for answer in [Answer::Star, Answer::Never] {
            let mut t = used(3);
            t.answer(answer, START + 4 * DAY);
            assert!(!t.should_ask(START + 400 * DAY));
        }
    }

    #[test]
    fn the_first_run_is_kept() {
        let mut t = Tips::default();
        t.started(START);
        t.started(START + DAY);
        assert_eq!(t.first_run, START);
    }

    #[test]
    fn the_file_is_in_the_state_folder() {
        assert!(Tips::path().starts_with(config::state_dir()));
    }

    #[test]
    fn answers_read_as_the_ui_sends_them() {
        let a: Vec<Answer> = serde_json::from_str(r#"["star","not-now","never"]"#).unwrap();
        assert_eq!(a, [Answer::Star, Answer::NotNow, Answer::Never]);
    }
}
