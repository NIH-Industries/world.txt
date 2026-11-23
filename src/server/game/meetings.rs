//! Meeting spots: somewhere to gather, with links for whatever's going on
//! (a call, a list, a song). A meeting table (or a gazebo) holds them; 'meet' hands it to you and
//! tells everyone there you've gone in. Events can carry a link too.

use super::Game;
use crate::protocol::{Col, Line};
use crate::server::db::EventRow;

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

/// `put [the call](https://…) on the table` → (its links, "the table"),
/// read from what comes after `put`. Nothing to put without a link, so
/// `put shell in the jar` stays a display case's.
pub fn put_words(rest: &str) -> Option<(String, String)> {
    let (words, links) = split_link(rest);
    let links = links?;
    let padded = format!(" {}", words.to_lowercase());
    let target = [" onto ", " on ", " at ", " to "]
        .iter()
        .filter_map(|p| padded.find(p).map(|at| at + p.len()))
        .min()
        .map_or(words.as_str(), |at| &words[at - 1..]);
    Some((links, target.trim().to_string()))
}

/// `take the call off the table` → ("the call", "the table").
pub fn take_words(rest: &str) -> Option<(String, String)> {
    let lower = rest.to_lowercase();
    let (at, len) = [" off of ", " off ", " from ", " out of "]
        .iter()
        .filter_map(|p| lower.find(p).map(|at| (at, p.len())))
        .min_by_key(|&(at, len)| (at, std::cmp::Reverse(len)))?;
    Some((
        rest[..at].trim().to_string(),
        rest[at + len..].trim().to_string(),
    ))
}

/// A link's words (or its address, if it has none) and its address.
fn label_url(link: &str) -> (String, String) {
    match crate::links::pieces(link).first() {
        Some(crate::links::Piece::Link { label, url }) => (label.to_string(), url.to_string()),
        _ => (link.to_string(), link.to_string()),
    }
}

/// "The Call link" → "call", for matching what someone takes off.
fn plain(words: &str) -> String {
    let w = words.trim().to_lowercase();
    let w = crate::server::world::strip_article(&w).trim();
    w.strip_suffix(" link").unwrap_or(w).trim().to_string()
}

/// Take the link called `what` (by its words or its address) out of
/// `links`: what's left (None when nothing is), and what was taken. `all`,
/// `links` and the like take everything; a bare `link` takes the only one.
/// None if no link (or more than one) is called that.
pub fn without(links: &str, what: &str) -> Option<(Option<String>, Vec<String>)> {
    let all = list(links);
    let w = plain(what);
    let everything = matches!(
        w.as_str(),
        "all" | "everything" | "links" | "all links" | "all the links" | "all of them" | "them"
    ) || (matches!(w.as_str(), "link" | "it" | "") && all.len() == 1);
    if everything {
        return (!all.is_empty()).then_some((None, all));
    }
    let named = |exact: bool| -> Vec<String> {
        all.iter()
            .filter(|l| {
                let (label, url) = label_url(l);
                let label = plain(&label);
                if exact {
                    label == w || url.eq_ignore_ascii_case(what.trim())
                } else {
                    label.contains(&w) || url.to_lowercase().contains(&w)
                }
            })
            .cloned()
            .collect()
    };
    let mut gone = named(true);
    if gone.is_empty() {
        gone = named(false);
    }
    if gone.len() != 1 {
        return None;
    }
    let left: Vec<String> = all.into_iter().filter(|l| !gone.contains(l)).collect();
    Some(((!left.is_empty()).then(|| left.join(" ")), gone))
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

    /// For `look`: what's on the table here, and the links events here bring.
    pub(super) fn meeting_lines_here(&self, id: u64) -> Vec<Line> {
        let Some(p) = self.players.get(&id) else {
            return vec![];
        };
        let Some((thing, name, _)) = self.meeting_spot_at(p.x, p.y) else {
            return vec![];
        };
        let the_spot = super::capitalise(&super::the(&name));
        let mut lines = match self.meeting_links.get(&thing) {
            Some(link) => vec![
                Line::new()
                    .c(Col::Cyan, format!("🔗 On {}: ", super::the(&name)))
                    .c(Col::Cyan, shown(link))
                    .c(Col::Dim, " (meet to join in)"),
            ],
            None => vec![Line::col(
                Col::Dim,
                format!(
                    "🔗 {the_spot} is bare. Leave a link for whatever happens here: put [the call](https://…) on the table"
                ),
            )],
        };
        for (event, when, own) in self.own_links_at(id, p.x, p.y) {
            lines.push(
                Line::new()
                    .c(
                        Col::Cyan,
                        format!("   📅 {event} ({when}) brings its own too: "),
                    )
                    .c(Col::Cyan, shown(&own)),
            );
        }
        if self.meeting_links.contains_key(&thing) {
            lines.push(Line::col(
                Col::Dim,
                if matches!(self.board_here(p.x, p.y), Some((_, false))) {
                    "   Every event here brings the table's links along, and reminds whoever follows the noticeboard here."
                } else {
                    "   Every event here brings the table's links along."
                },
            ));
        }
        lines
    }

    /// The gathering table someone means by `words` (`the table`, its
    /// name, or nothing at all) at their spot, or else the event they name.
    fn link_target(&self, id: u64, words: &str) -> anyhow::Result<Option<Target>> {
        let Some(p) = self.players.get(&id) else {
            return Ok(None);
        };
        let w = plain(words);
        let table_word = matches!(
            w.as_str(),
            "" | "table" | "here" | "spot" | "gazebo" | "gathering table" | "gathering spot"
        );
        if let Some((thing, name, maker)) = self.meeting_spot_at(p.x, p.y)
            && (table_word || name.to_lowercase().contains(&w))
        {
            return Ok(Some(Target::Table(thing, name, maker)));
        }
        if table_word {
            return Ok(None);
        }
        Ok(self.pick_event(words)?.map(Target::Event))
    }

    /// Whether someone may change the links on a table: whoever made it,
    /// the home's owner, a keeper, or anyone at all for the island's own.
    /// Says why not, if not.
    fn may_set_table(&self, id: u64, name: &str, maker: &str) -> bool {
        let user = self.user(id);
        let home_owner = self
            .players
            .get(&id)
            .and_then(|p| self.world.home_at(p.x, p.y))
            .is_some_and(|h| h.owner == user);
        // The island's own tables (the one on the village green) are everyone's.
        let ok = maker == user || home_owner || maker == super::green::BY || self.is_keeper(&user);
        if !ok {
            self.say1(
                id,
                Line::plain(format!(
                    "That's {}'s {}, so only they (or a keeper) can change what's on it. You can still meet to join in!",
                    self.world.nick(maker),
                    super::the(name).trim_start_matches("the ")
                )),
            );
        }
        ok
    }

    /// Keep these links on a table (None: nothing on it).
    fn store_table_links(
        &mut self,
        id: u64,
        thing: i64,
        link: Option<String>,
    ) -> anyhow::Result<()> {
        let user = self.user(id);
        self.world
            .db
            .set_meeting_link(thing, link.as_deref(), &user, super::boards::now())?;
        match link {
            Some(l) => self.meeting_links.insert(thing, l),
            None => self.meeting_links.remove(&thing),
        };
        Ok(())
    }

    /// `put [the call](https://…) on the table` (or `on Tea party`): add
    /// links to a gathering table or an event, three at most.
    pub(super) fn cmd_put_links(&mut self, id: u64, links: &str, onto: &str) -> anyhow::Result<()> {
        let Some(target) = self.link_target(id, onto)? else {
            let msg = if plain(onto).is_empty() || plain(onto) == "table" {
                "Links go on a gathering table (make a gathering table), or on an event: put https://… on Tea party".to_string()
            } else {
                format!(
                    "There's no table here or event called {onto} to put them on. ('events' lists what's planned, by name)"
                )
            };
            self.say1(id, Line::plain(msg));
            return Ok(());
        };
        let (what, had) = match &target {
            Target::Table(thing, name, _) => {
                (super::the(name), self.meeting_links.get(thing).cloned())
            }
            Target::Event(e) => (e.name.clone(), e.link.clone()),
        };
        let all = had
            .as_deref()
            .map_or(links.to_string(), |h| merged(h, links));
        if list(&all).len() > MAX_LINKS {
            let first = had
                .as_deref()
                .map(list)
                .and_then(|l| l.into_iter().next())
                .map(|l| label_url(&l).0)
                .unwrap_or_default();
            self.say1(id, Line::plain(format!(
                "{} can only hold three links, and it's got {} already. Take one off first: take {first} off {}",
                super::capitalise(&what),
                shown(had.as_deref().unwrap_or_default()),
                if matches!(target, Target::Table(..)) { "the table".to_string() } else { what.clone() },
            )));
            return Ok(());
        }
        match target {
            Target::Table(thing, name, maker) => {
                if !self.may_set_table(id, &name, &maker) {
                    return Ok(());
                }
                self.store_table_links(id, thing, Some(all.clone()))?;
                self.say_table_now(id, &name, Some(&all));
                let (x, y, who) = self.here(id).unwrap_or_default();
                let user = self.user(id);
                self.tell_here_from(
                    x,
                    y,
                    id,
                    &user,
                    Line::col(
                        Col::Cyan,
                        format!(
                            "🔗 {who} leaves a link on {}. (meet to join in)",
                            super::the(&name)
                        ),
                    ),
                );
            }
            Target::Event(e) => {
                if self.may_change_event_links(id, &e) {
                    self.store_event_links(id, &e, Some(all))?;
                }
            }
        }
        Ok(())
    }

    /// `take the call off the table` (or `off Tea party`; `take all off`
    /// takes everything). None when it isn't about links after all, like
    /// `take shell out of the jar` or `remove Poppy from Tea party`.
    pub(super) fn cmd_take_links(
        &mut self,
        id: u64,
        what: &str,
        from: &str,
        table_only: bool,
    ) -> Option<anyhow::Result<()>> {
        let target = match self.link_target(id, from) {
            Ok(t) => t?,
            Err(e) => return Some(Err(e)),
        };
        match target {
            Target::Table(thing, name, maker) => {
                let the_spot = super::capitalise(&super::the(&name));
                let Some(links) = self.meeting_links.get(&thing).cloned() else {
                    self.say1(
                        id,
                        Line::plain(format!(
                            "{the_spot} is already bare: there are no links on it."
                        )),
                    );
                    return Some(Ok(()));
                };
                let Some((left, gone)) = without(&links, what) else {
                    self.say1(
                        id,
                        Line::plain(format!(
                            "{the_spot} has no link called {what}. On it: {}",
                            shown(&links)
                        )),
                    );
                    return Some(Ok(()));
                };
                if !self.may_set_table(id, &name, &maker) {
                    return Some(Ok(()));
                }
                if let Err(e) = self.store_table_links(id, thing, left.clone()) {
                    return Some(Err(e));
                }
                self.say1(
                    id,
                    Line::col(
                        Col::Cyan,
                        format!(
                            "🔗 You take {} off {}.",
                            shown(&gone.join(" ")),
                            super::the(&name)
                        ),
                    ),
                );
                self.say_table_now(id, &name, left.as_deref());
                Some(Ok(()))
            }
            Target::Event(_) if table_only => None,
            Target::Event(e) => {
                let own = e.link.as_deref().and_then(|l| without(l, what));
                let Some((left, gone)) = own else {
                    // It might be the table's, coming along with the event.
                    let table = self.meeting_link_at(e.x, e.y)?;
                    without(&table, what)?;
                    self.say1(id, Line::plain(format!("That link comes from the gathering table where {} happens. Take it off the table there and it's gone from every event there.", e.name)));
                    return Some(Ok(()));
                };
                if !self.may_change_event_links(id, &e) {
                    return Some(Ok(()));
                }
                self.say1(
                    id,
                    Line::col(
                        Col::Cyan,
                        format!("🔗 You take {} off {}.", shown(&gone.join(" ")), e.name),
                    ),
                );
                Some(self.store_event_links(id, &e, left))
            }
        }
    }

    /// What's on a table now, after a change.
    fn say_table_now(&self, id: u64, name: &str, link: Option<&str>) {
        let the_spot = super::capitalise(&super::the(name));
        let line = match link {
            Some(l) => Line::new()
                .c(Col::Cyan, format!("🔗 On {} now: ", super::the(name)))
                .c(Col::Cyan, shown(l))
                .c(
                    Col::Dim,
                    ". Anyone here can meet to join in, and every event here brings them along.",
                ),
            None => Line::col(Col::Cyan, format!("🔗 {the_spot} is bare now.")),
        };
        self.say1(id, line);
    }

    /// `meet`: join in with whatever's on here (an event's own links first,
    /// then the table's). `meet https://…` and `meet off` still work as the
    /// old way of putting links on the table and clearing it.
    pub(super) fn cmd_meet(&mut self, id: u64, rest: &str) -> anyhow::Result<()> {
        let Some((x, y, name)) = self.here(id) else {
            return Ok(());
        };
        let rest = rest.trim();
        if matches!(
            rest.to_lowercase().as_str(),
            "off" | "clear" | "none" | "stop"
        ) {
            return self.cmd_take_links(id, "all", "", true).unwrap_or_else(|| {
                self.say1(id, Line::plain("There's no gathering table here to clear."));
                Ok(())
            });
        }
        if !rest.is_empty() {
            let (_, Some(link)) = split_link(rest) else {
                self.say1(
                    id,
                    Line::plain("That doesn't look like a link. Links start https://, like put https://… on the table"),
                );
                return Ok(());
            };
            // The old way replaced everything on the table.
            if let Some((thing, spot_name, maker)) = self.meeting_spot_at(x, y) {
                if !self.may_set_table(id, &spot_name, &maker) {
                    return Ok(());
                }
                self.store_table_links(id, thing, None)?;
            }
            return self.cmd_put_links(id, &link, "");
        }
        let spot = self.meeting_spot_at(x, y);
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
                "There's nothing on the table to join yet. Leave a link for whatever happens here: put [the call](https://…) on the table"
            } else {
                "There's nothing to join here. Gatherings happen at a gathering table: make a gathering table, then put [the call](https://…) on the table"
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
        Ok(())
    }
}

/// Where links go: a gathering table (its id, name and maker), or an event.
enum Target {
    Table(i64, String, String),
    Event(EventRow),
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
    fn reads_what_goes_on_what() {
        assert_eq!(
            put_words("[the call](https://meet.example.com/a) on the table"),
            Some((
                "[the call](https://meet.example.com/a)".into(),
                "the table".into()
            ))
        );
        assert_eq!(
            put_words("https://a.example.com/x https://b.example.com/y onto Tea party"),
            Some((
                "https://a.example.com/x https://b.example.com/y".into(),
                "Tea party".into()
            ))
        );
        assert_eq!(
            put_words("https://a.example.com/x"),
            Some(("https://a.example.com/x".into(), "".into())),
            "on the table here"
        );
        assert_eq!(
            put_words("[the zoom](https://z.example.com/1) on Fish on Fridays"),
            Some((
                "[the zoom](https://z.example.com/1)".into(),
                "Fish on Fridays".into()
            )),
            "the first 'on' after the link"
        );
        assert_eq!(put_words("shell in the jar"), None, "no link, a case's");
        assert_eq!(put_words("on my hat"), None);
    }

    #[test]
    fn reads_what_comes_off_what() {
        assert_eq!(
            take_words("the call off the table"),
            Some(("the call".into(), "the table".into()))
        );
        assert_eq!(
            take_words("snack list off of Tea party"),
            Some(("snack list".into(), "Tea party".into()))
        );
        assert_eq!(
            take_words("shell out of the jar"),
            Some(("shell".into(), "the jar".into()))
        );
        assert_eq!(take_words("the call"), None);
    }

    #[test]
    fn takes_one_link_off_by_its_words_or_address() {
        let links = "[the call](https://meet.example.com/a) [snack list](https://example.com/snacks) https://example.com/map";
        let (left, gone) = without(links, "the call").unwrap();
        assert_eq!(gone, ["[the call](https://meet.example.com/a)"]);
        assert_eq!(
            left.as_deref(),
            Some("[snack list](https://example.com/snacks) https://example.com/map")
        );
        assert_eq!(
            without(links, "Snack List link").unwrap().1,
            ["[snack list](https://example.com/snacks)"]
        );
        assert_eq!(
            without(links, "https://example.com/map").unwrap().1,
            ["https://example.com/map"]
        );
        assert_eq!(
            without(links, "snack").unwrap().1.len(),
            1,
            "a bit of its words is enough"
        );
        assert!(without(links, "example").is_none(), "that's more than one");
        assert!(without(links, "Poppy").is_none());
        assert!(without(links, "link").is_none(), "which one?");
        assert_eq!(without(links, "all"), Some((None, list(links))));
        assert_eq!(without(links, "the links").unwrap().0, None);
        assert_eq!(
            without("https://example.com/map", "link"),
            Some((None, vec!["https://example.com/map".to_string()])),
            "the only one"
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
