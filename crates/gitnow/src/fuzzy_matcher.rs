use nucleo_matcher::{pattern::Pattern, Matcher};

use crate::app::App;

pub struct FuzzyMatcher {}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn match_pattern<'a>(&self, pattern: &'a str, items: &'a [&'a str]) -> Vec<&'a str> {
        let pat = Pattern::new(
            &pattern,
            nucleo_matcher::pattern::CaseMatching::Ignore,
            nucleo_matcher::pattern::Normalization::Smart,
            nucleo_matcher::pattern::AtomKind::Fuzzy,
        );
        let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
        let res = pat.match_list(items, &mut matcher);

        res.into_iter().map(|((item, _))| *item).collect::<Vec<_>>()
    }
}

pub trait FuzzyMatcherApp {
    fn fuzzy_matcher(&self) -> FuzzyMatcher;
}

impl FuzzyMatcherApp for &'static App {
    fn fuzzy_matcher(&self) -> FuzzyMatcher {
        FuzzyMatcher::new()
    }
}
