//! Meeting spots: somewhere to gather, with links for whatever's going on
//! (a call, a list, a song). A meeting table (or a gazebo) holds them; 'meet' hands it to you and
//! tells everyone there you've gone in. Events can carry a link too.

use super::Game;
use crate::protocol::{Col, Line};

const KIND: &str = "meeting";
const MAX_LINK: usize = 300;

pub fn kind() -> &'static str {
    KIND
}

pub fn is_meeting_spot(name: &str) -> bool {
    let n = name.to_lowercase();
    [
        "meeting",
        "gazebo",
        "gathering table",
        "gathering spot",
        "gathering place",
    ]
    .iter()
    .any(|w| n.contains(w))
}

/// A call link worth keeping: https, one piece, not too long.
pub fn clean_link(raw: &str) -> Option<String> {
    let url = raw.trim().trim_end_matches(['.', ',', '!', '?', ';', ')']);
    let ok = url.starts_with("https://")
        && url.len() > "https://".len() + 3
        && url.chars().count() <= MAX_LINK
        && !url
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | '"' | '\''));
    ok.then(|| url.to_string())
}

/// How many links one event or meeting spot keeps (a call, a board, notes).
const MAX_LINKS: usize = 3;

/// Take the links out of some words, bare (`https://…`) or with words
/// (`[the snack list](https://…)`): "Standup [the call](https://meet…)
/// https://trello…" → ("Standup", "[the call](https://meet…) https://trello…").
/// Several are kept together, a space apart.
pub fn split_link(text: &str) -> (String, Option<String>) {
    let (rest, links) = take_links(text, MAX_LINKS);
    (rest, (!links.is_empty()).then(|| links.join(" ")))
}

/// The links kept together, one each: "[the call](https://a)", "https://b".
pub fn list(links: &str) -> Vec<String> {
    take_links(links, usize::MAX).1
}

/// Links kept together, for reading: "the call · https://b" (the browser
/// makes "the call" clickable; the terminal shows its address beside it).
pub fn shown(links: &str) -> String {
    list(links).join(" · ")
}

/// Two lots of links as one, in order, without the same address twice.
pub fn merged(first: &str, then: &str) -> String {
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for l in list(first).into_iter().chain(list(then)) {
        let url = match crate::links::pieces(&l).first() {
            Some(crate::links::Piece::Link { url, .. }) => url.to_string(),
            _ => l.clone(),
        };
        if !seen.contains(&url) {
            seen.push(url);
            out.push(l);
        }
    }
    out.join(" ")
}

/// Up to `max` links out of `text`, and what's left once they're gone.
fn take_links(text: &str, max: usize) -> (String, Vec<String>) {
    let mut links: Vec<String> = Vec::new();
    let mut rest = String::new();
    for piece in crate::links::pieces(text) {
        match piece {
            crate::links::Piece::Link { label, url } => match clean_link(url) {
                Some(url) if links.len() < max => {
                    links.push(format!("[{label}]({url})"));
                    rest.push(' ');
                }
                _ => rest.push_str(&format!("[{label}]({url})")),
            },
            crate::links::Piece::Text(t) => {
                for w in t.split_inclusive(char::is_whitespace) {
                    if links.len() < max
                        && w.starts_with("https://")
                        && let Some(l) = clean_link(w)
                    {
                        links.push(l);
                        rest.push(' ');
                    } else {
                        rest.push_str(w);
                    }
                }
            }
        }
    }
    (rest.split_whitespace().collect::<Vec<_>>().join(" "), links)
}

impl Game {
    pub(super) fn load_meetings(&mut self) {
        match self.world.db.meeting_links() {
            Ok(links) => self.meeting_links = links.into_iter().collect(),
            Err(e) => eprintln!("couldn't load meeting links: {e}"),
        }
    }

    /// The meeting spot at (x, y), if there is one.
    fn meeting_spot_at(&self, x: i64, y: i64) -> Option<(i64, String, String)> {
        self.world
            .things_at(x, y)
            .find(|t| t.kind.as_deref() == Some(KIND))
            .map(|t| (t.id, t.name.clone(), t.maker.clone()))
    }

    pub(super) fn meeting_here(&self, id: u64) -> bool {
        self.players
            .get(&id)
            .is_some_and(|p| self.meeting_spot_at(p.x, p.y).is_some())
    }

    /// The standing call link at (x, y), from a meeting spot there.
    pub(super) fn meeting_link_at(&self, x: i64, y: i64) -> Option<String> {
        let (thing, _, _) = self.meeting_spot_at(x, y)?;
        self.meeting_links.get(&thing).cloned()
    }

    /// For `look`: the call link here, if any.
    pub(super) fn meeting_lines_here(&self, id: u64) -> Vec<Line> {
        let Some(p) = self.players.get(&id) else {
            return vec![];
        };
        let Some((thing, name, _)) = self.meeting_spot_at(p.x, p.y) else {
            return vec![];
        };
        match self.meeting_links.get(&thing) {
            Some(link) => vec![
                Line::new()
                    .c(
                        Col::Cyan,
                        format!(
                            "🔗 {} has {}: ",
                            super::capitalise(&super::the(&name)),
                            if list(link).len() > 1 {
                                "links"
                            } else {
                                "a link"
                            }
                        ),
                    )
                    .c(Col::Cyan, shown(link))
                    .c(Col::Dim, " (meet to get them)"),
                Line::col(
                    Col::Dim,
                    if matches!(self.board_here(p.x, p.y), Some((_, false))) {
                        "   Events here bring them along, and remind whoever follows the noticeboard here."
                    } else {
                        "   Events here bring them along, after their own."
                    },
                ),
            ],
            None => vec![Line::col(
                Col::Dim,
                "🔗 Nobody's left any links here yet (meet https://…)",
            )],
        }
    }

    /// `meet` (go into the call here), `meet https://…` (set the spot's
    /// link) and `meet off` (clear it).
    pub(super) fn cmd_meet(&mut self, id: u64, rest: &str) -> anyhow::Result<()> {
        let Some((x, y, name)) = self.here(id) else {
            return Ok(());
        };
        let rest = rest.trim();
        let spot = self.meeting_spot_at(x, y);
        if rest.is_empty() {
            // An event on now comes first (its own call), then the table's.
            let table = spot
                .as_ref()
                .and_then(|s| self.meeting_links.get(&s.0).cloned());
            let link = match (self.event_link_now(x, y), table) {
                (Some(e), Some(t)) => Some(merged(&e, &t)),
                (e, t) => e.or(t),
            };
            let Some(link) = link else {
                let msg = if spot.is_some() {
                    "There are no links here yet. Leave some for whatever happens here: meet https://…"
                } else {
                    "There's nothing to join here. Gatherings happen at a gathering table: make a gathering table, then meet https://…"
                };
                self.say1(id, Line::plain(msg));
                return Ok(());
            };
            self.say1(
                id,
                Line::new()
                    .c(Col::Cyan, "🔗 Here you go: ")
                    .b(Col::Cyan, shown(&link))
                    .c(
                        Col::Dim,
                        " (the island will be right here when you're back)",
                    ),
            );
            let user = self.user(id);
            self.tell_here_from(
                x,
                y,
                id,
                &user,
                Line::col(Col::Cyan, format!("🔗 {name} heads off to the gathering.")),
            );
            return Ok(());
        }
        let Some((thing, spot_name, maker)) = spot else {
            self.say1(
                id,
                Line::plain(
                    "Links live on gathering spots. Make one here first: make a gathering table",
                ),
            );
            return Ok(());
        };
        let user = self.user(id);
        let home_owner = self.world.home_at(x, y).is_some_and(|h| h.owner == user);
        // The island's own tables (the one on the village green) are everyone's.
        let everyones = maker == super::green::BY;
        if maker != user && !home_owner && !everyones && !self.is_keeper(&user) {
            self.say1(
                id,
                Line::plain(format!(
                    "Only whoever made {} (or a keeper) can change its links.",
                    super::the(&spot_name)
                )),
            );
            return Ok(());
        }
        let the_spot = super::the(&spot_name);
        if matches!(
            rest.to_lowercase().as_str(),
            "off" | "clear" | "none" | "stop"
        ) {
            self.world
                .db
                .set_meeting_link(thing, None, &user, super::boards::now())?;
            self.meeting_links.remove(&thing);
            self.say1(
                id,
                Line::col(
                    Col::Cyan,
                    format!("🔗 {} has no links now.", super::capitalise(&the_spot)),
                ),
            );
            return Ok(());
        }
        let (_, Some(link)) = split_link(rest) else {
            self.say1(
                id,
                Line::plain("That doesn't look like a link. Links need to start https://"),
            );
            return Ok(());
        };
        self.world
            .db
            .set_meeting_link(thing, Some(&link), &user, super::boards::now())?;
        self.meeting_links.insert(thing, link.clone());
        self.say1(
            id,
            Line::new()
                .c(
                    Col::Cyan,
                    format!("🔗 {} now holds ", super::capitalise(&the_spot)),
                )
                .c(Col::Cyan, shown(&link))
                .c(Col::Dim, ". Anyone here can type meet to get them."),
        );
        self.tell_here_from(
            x,
            y,
            id,
            &user,
            Line::col(
                Col::Cyan,
                format!("🔗 {name} leaves some links at {the_spot}. (meet to get them)"),
            ),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_link_out_of_an_events_name() {
        let (name, link) = split_link("Standup https://meet.google.com/abc-defg-hij");
        assert_eq!(
            (name.as_str(), link.as_deref()),
            ("Standup", Some("https://meet.google.com/abc-defg-hij"))
        );
        let (name, link) = split_link("Retro (https://zoom.us/j/123) with cake");
        assert_eq!(link, None, "a bracketed link isn't a clean word");
        assert!(name.contains("Retro"));
        let (name, link) =
            split_link("Standup https://meet.google.com/abc https://trello.com/b/xyz/team");
        assert_eq!(
            (name.as_str(), link.as_deref()),
            (
                "Standup",
                Some("https://meet.google.com/abc https://trello.com/b/xyz/team")
            )
        );
        assert_eq!(
            shown(&link.unwrap()),
            "https://meet.google.com/abc · https://trello.com/b/xyz/team"
        );
        let (_, link) = split_link("x https://a.bc/1 https://a.bc/2 https://a.bc/3 https://a.bc/4");
        assert_eq!(link.unwrap().split(' ').count(), 3, "three at most");
        let (name, link) = split_link("Beach party");
        assert_eq!((name.as_str(), link), ("Beach party", None));
    }

    #[test]
    fn links_can_have_words() {
        let (name, link) = split_link(
            "Tea party [the call](https://meet.example.com/abc) [snack list](https://example.com/snacks) https://example.com/map",
        );
        assert_eq!(name, "Tea party");
        let link = link.unwrap();
        assert_eq!(
            list(&link),
            [
                "[the call](https://meet.example.com/abc)",
                "[snack list](https://example.com/snacks)",
                "https://example.com/map",
            ]
        );
        assert_eq!(
            shown(&link),
            "[the call](https://meet.example.com/abc) · [snack list](https://example.com/snacks) · https://example.com/map"
        );
        let (name, link) = split_link("Picnic [the map](http://example.com/map)");
        assert_eq!(link, None, "still https only");
        assert_eq!(name, "Picnic [the map](http://example.com/map)");
        let (name, link) = split_link("Picnic [the map](https://example.com/map) by the pond");
        assert_eq!(name, "Picnic by the pond");
        assert_eq!(link.as_deref(), Some("[the map](https://example.com/map)"));
    }

    #[test]
    fn an_events_links_then_the_tables() {
        assert_eq!(
            merged(
                "[the call](https://meet.example.com/a) https://example.com/board",
                "[the board](https://example.com/board) https://example.com/snacks",
            ),
            "[the call](https://meet.example.com/a) https://example.com/board https://example.com/snacks"
        );
    }

    #[test]
    fn only_tidy_https_links() {
        assert_eq!(
            clean_link("https://meet.google.com/abc.").as_deref(),
            Some("https://meet.google.com/abc")
        );
        assert_eq!(clean_link("http://meet.google.com/abc"), None);
        assert_eq!(clean_link("https://"), None);
        assert_eq!(clean_link("https://a.b/\"><script>"), None);
        assert_eq!(
            clean_link(&format!("https://x.y/{}", "a".repeat(400))),
            None
        );
    }

    #[test]
    fn knows_a_meeting_spot() {
        assert!(is_meeting_spot("a round meeting table"));
        assert!(is_meeting_spot("a little Gazebo"));
        assert!(is_meeting_spot("a gathering table"));
        assert!(
            !is_meeting_spot("a gathering basket"),
            "gathering finds is something else"
        );
        assert!(!is_meeting_spot("a bench"));
    }
}
