//! What the page's link asks for: the eclipse to open and whether to fly down to it.

#[cfg(test)]
mod tests;

/// The query value of a link that flies down to the typical place for the eclipse.
const STAND_GREATEST: &str = "greatest";
/// How long a link shows the whole eclipse before flying down, seconds.
const OVERVIEW_S: f64 = 2.0;

/// A page's query.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PageLink {
    /// The `YYYY-MM-DD` date of the eclipse, or `-YYYY-MM-DD` before year 0.
    pub(crate) date: Option<String>,
    /// Whether the eye stands at greatest eclipse, or under the Moon of a lunar one.
    pub(crate) stand: bool,
}

impl PageLink {
    /// The link a query string names; parts it does not know are ignored.
    pub(crate) fn parse(search: &str) -> Self {
        let pairs = || search.trim_start_matches('?').split('&');
        Self {
            date: pairs()
                .find_map(|pair| pair.strip_prefix("date="))
                .filter(|date| date.trim_start_matches('-').len() == 10)
                .map(str::to_owned),
            stand: pairs().any(|pair| pair.strip_prefix("stand=") == Some(STAND_GREATEST)),
        }
    }

    /// This link as a query string, empty when it names nothing.
    pub(crate) fn query(&self) -> String {
        let date = self.date.as_ref().map(|date| format!("date={date}"));
        let stand = self.stand.then(|| format!("stand={STAND_GREATEST}"));
        let parts: Vec<String> = date.into_iter().chain(stand).collect();
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }
}

/// The overview a link's flight waits out, which runs from the first frame on screen.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OpeningFlight {
    overview_from_s: Option<f64>,
}

impl OpeningFlight {
    /// The seconds of overview left at `now_s`, or `None` once it is time to fly.
    pub(crate) fn remaining(&mut self, now_s: f64) -> Option<f64> {
        let from = *self.overview_from_s.get_or_insert(now_s);
        let left = OVERVIEW_S - (now_s - from);
        (left > 0.0).then_some(left)
    }
}
