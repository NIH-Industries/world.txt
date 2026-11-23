# Events, gatherings and noticeboards

Everything for getting your people together: events (for the whole island or
just one spot), gathering tables with links, and noticeboards that work like
group chats. In the world, `help gatherings` has the short version.

## How it fits together

Everything belongs to a spot. On one spot, a **noticeboard** says who (its
followers hear about events there, like a group chat), a **gathering table**
says what (its links, a call, a list, come along to every event there), and
an **event** says when. None needs the others, and the island tells you how
they're teaming up as you go.

## Events

Keepers plan events where they're standing: `event sat 4pm Beach party` (or
`tomorrow 10:30am`, `noon`...). Times are your own: browsers (and the terminal
client) tell the world which timezone they're in, so 2pm means 2pm where you
are, and everyone sees event times in theirs. Notes everyone reads alike (the
Island Journal, a board) name the zone: "2:00pm AEST". The side panel shows your real
date and time (🕰 … your time) under the island's own clock, and `events` starts
with it too. In the browser, everyone else out in the world is listed at the foot of
the side panel with their local time (tap a name to talk to them, and whoever's
on your spot is marked "here"); `who` shows what time it is for everyone online
("3:12am Fri in London"), and `time Poppy` tells you Poppy's, from where they
last were if they're away. It goes in the Island Journal, everyone hears a
reminder ten minutes before and a shout when it starts, `events` lists what's
coming up, and `go event` walks you there. `cancel Tea party` for its host or a keeper.
Wherever a command takes an event, you can call it by a name only one event
has (`cancel Tea party`), by its number from `events` (`cancel event 2`), or
by nothing at all when there's just one event. Anything else is answered with a question,
never a guess.

Events can also belong to just one place, so a little get-together doesn't ping
the whole island: `event here fri 4pm Tea party`. Anyone can plan one. Only its host,
people who've said `going Tea party` (`not going Tea party` drops
out), followers of a noticeboard on that spot, and anyone within ten steps hear
it announced, reminded and starting. It's posted on that noticeboard rather
than the Island Journal, and `events` marks it 📍 with who's going.
Events can repeat: `event here fridays 5pm Frog choir` (also `weekdays`,
`weekends`, `daily`, or `every mon wed`). It stays at 5pm in the planner's
time, even across daylight saving, moves on to the next one an hour after each
starts, and `events` shows it as "🔁 every fri · next Fri 6 Nov, 5:00pm". Who's
going, the board's followers and its links carry over; `cancel Frog choir` stops
the series. On a repeating event, `not going Frog choir` skips just the next one
(`going Frog choir` un-skips it), and `leave event Frog choir` drops out for good.
`event here sun 10am Pancakes with Poppy, Sam and Lu` signs them up as going
too: they're told straight away, or find a note waiting when they next arrive.
Later, `add Poppy, Sam to Pancakes` signs more people up (its host or anyone
going can), and `remove Poppy from Pancakes` takes someone off (the host or a
keeper; anyone can drop themselves with `not going Pancakes`).
A place event's links are only shown to people who are in on it (going,
following its board, or standing nearby); everyone else sees that it has one.
Its host or a keeper can change an event's own links any time without
cancelling it: `put [the call](https://…) on Tea party` adds one, and
`take the call off Tea party` takes it back off (`take all off Tea party`
takes the lot). Who's going and how it repeats stay just as they were. (The
older `event links 1 …` and `event links 1 off` still work.)

## Gatherings

Somewhere to gather, with links for whatever's going on: a call, a list, a song.
`help gatherings` walks through the whole recipe, and the island nudges you along.
`make a gathering table` (or a gazebo) makes a gathering spot. Links are
things you put on it, like finds in a display case: its maker, the home's
owner or a keeper puts them there with
`put [the call](https://example.com/gathering) on the table` (up to three; a
bare `https://…` works too). Words in `[ ]` show instead of the address:
"the call · snack list", each clickable in the browser (the terminal shows the
address beside it). `take the call off the table` takes one back off, by its
words or its address, and `clear the table` takes them all. Anyone standing
there sees what's on it when they look, and `meet` joins in: it hands you
the links and tells everyone there you've headed off to the gathering. (The
older `meet https://…` and `meet off` still work, and replace everything on
the table.)

Events can have links too, of their own: `event here fri 4pm Tea party
[the zoom](https://…)` when planning, or `put [the zoom](https://…) on Tea party`
later. An event at a gathering table brings whatever's on the table along,
after its own, so each event can have its own call while the table holds what
they all share, like the team's board. The table's links aren't copied, so
change the table and every event there has the new ones straight away.
Looking at the table shows both: what's on it, and which events there bring
links of their own.
They show in `events`, the reminder and the "starting now" shout, and `meet`
at an event's spot while it's on hands them over, each clickable on its own. In the browser, https links anywhere (lines and board
notes) open in a new tab.

## Noticeboards

`make a noticeboard` puts one up (shown as `▣` on the map). It works like a group
chat that belongs to that spot:

- `read` shows the latest notes and `read more` scrolls back. In the browser, a
  board opens as a chat panel that loads older notes as you scroll up.
- `post see you at sunset!` leaves a note. Everyone standing there sees it, and
  posting **follows** the board, so you hear about new notes wherever you are.
  `follow` / `unfollow` to change that, and `boards` lists the ones you follow
  (`unfollow 2` leaves one from anywhere).
- Links can have words: `post [the snack list](https://example.com/snacks)`
  shows "the snack list", clickable in the browser (the terminal shows the
  address beside it). That's the only formatting there is.
- A board's followers are its group, like a group chat: anyone who follows it
  (or made it, or a keeper) can `add Poppy, Sam to board`, and they're told, or
  find a note waiting. They hear its notes and the place events planned there.
- **Hooks** let outside tools post too (a doorbell, a smart kettle, a calendar). At a
  board, its maker or a keeper types `hook Doorbell` and is shown a secret address,
  once. Anything POSTed there lands on the board as a note from `🔔 Doorbell`, like
  any other post: plain text, or JSON with `text` (Slack style) or `content`
  (Discord style), trimmed to 280 characters. Slack-style links
  (`<https://…|the cat cam>`) come out as proper links.
  `curl -d 'ding dong 🚪' https://world.example.com/hook/<secret>`
  `hooks` lists a board's hooks and when they last posted, and `unhook Doorbell`
  stops one. It answers 200 when posted, 404 for a stopped or unknown hook,
  429 if one hook posts more than once every two seconds, and 413 past 16KB.
  Not on the Island Journal.
- **Progress bars:** `progress Pizza` at a board gives a hook of another kind.
  Instead of notes, it's sent how far along something is: `37/100`, `37%`, or
  `3 of 10 photos` (any words after the numbers become its label; otherwise
  it's called Pizza). Everyone who follows the board, or is within a few
  steps of it, sees the bar fill in their side panel, with its label on the
  right. Nothing is said in anyone's log. It can post as often as it likes:
  the bar moves on at most once a second, always to the latest number. When it's finished, `clear Pizza` at the board (or the tool posting
  `clear`) takes the bar away until it next says something, and `unhook
  Pizza` stops it for good.
  `curl -d '3 of 10 photos' https://world.example.com/hook/<secret>`
- **Status lights:** `status Surf` at a board gives a hook that's sent `up` or
  `down` (`ok`, `passed`, `failed`, `broken` and friends work too), with any
  words after it kept as a note: `down flat as a millpond`, or with a link,
  `down [the beach cam](https://surf.example.com/cam) says flat` (the side panel
  shows "the beach cam", clickable in the browser). Pingdom-style JSON
  (`current_state`) and Semaphore's notification webhook work as they are.
  Everyone who follows the board, or is
  near it, sees a green or red light in their side panel, and gets a message
  when it goes down or comes back up. Staying up says nothing, and the first
  word from something that's fine is quiet. A flapping check can't chatter:
  a light speaks at most once a minute, and only to say where it ended up.
  `clear Surf` hides it until it next says something.
  `curl -d 'down flat as a millpond' https://world.example.com/hook/<secret>`

The **Island Journal** is a special board at the spawn point. The first find of
every species each week goes in it automatically, with a shout-out to the whole
island, and people can post underneath. It starts a fresh page every Monday:
`archive` lists past weeks and `read week 2026-10-26` opens one. When the journal is first put up it fills in every earlier week from
everyone's personal journals; `world.txt backfill-journal` does the same for a
journal that already exists (it's safe to run again).

---
[← back to the README](../README.md)
