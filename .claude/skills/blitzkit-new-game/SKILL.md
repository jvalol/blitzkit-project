---
name: blitzkit-new-game
description: Set up a new game on blitzkit, inside ~/Developer/blitzkit-project. Covers the repo scaffold, the spec flow, the GitHub repo, and every place a game has to be registered (the project readme, the engine readme, and the docs site). Use when adding a new game or crate to the project, or when asking what a new game has to touch.
---

# Adding a game to blitzkit

A game is its own repo, cloned into `blitzkit-project/games`. The engine sits
beside that folder rather than in it.
Nothing in a `cargo run` reads prose, so the places a game has to be named are
the places it silently goes missing. Work the list.

## What no longer needs doing

`check-all`, `check-specs` and `check-listings` all read `./list-repos`, which
reads the disk. A crate with a `Cargo.toml` under `games/` is in the gate the
moment it exists. Do not add it to an array anywhere; there is no array.

This was not always true. securitysweep was in `check-all`'s list and never made
it into `check-specs`', so its specs went unchecked and the gate stayed green.

The arcade needs no edit either. It reads `games/` from the disk the same way,
so a new game gets a cabinet the moment its folder exists, and
`arcade::tests::it_knows_every_game_the_project_does` holds its list against
`./list-repos games` so the two cannot come apart.

## The repo itself

Copy the shape of the newest sibling rather than inventing one. cascada is the
most recent.

- `Cargo.toml`. `edition = "2018"`, `rust-version = "1.87"` (the engine's MSRV),
  `license = "MIT OR Apache-2.0"`, and `blitzkit` at the current minor version.
  The published crate, not a path: the `[patch.crates-io]` in
  `blitzkit-project/.cargo/config.toml` is what redirects it to the checkout, and
  that is the whole point. A game that names a path is not testing what a
  stranger gets.
- `LICENSE-MIT` and `LICENSE-APACHE`, copied from a sibling.
- `.gitignore`, copied from a sibling.
- `CLAUDE.md`. What the game is, which number it is, the build block, the spec
  flow, and a section per decision worth defending. The last part is what makes
  these files worth reading.
- `specs/TEMPLATE.md` and `specs/README.md`, copied, with the index table
  emptied. Then write `specs/0001-<name>.md` before any code.
- `README.md` and `src/main.rs`.
- `media/screenshot.png`, 1200 wide. check-listings fails without it, and
  without the site entry that shows it. Take it rather than asking: the
  `blitzkit-screenshot` skill says what the picture has to be and where it has
  to be named, and `mac-window-screenshot` says how to take one.

  Expect to stage the game first. A first frame rarely shows what a game is
  about: both lantern's and securitysweep's needed the state moved, a camera
  lifted over the yard, a candle put down and the player turned to look back
  at it.

First commit subject: `Schöpfung aus dem Nichts`. Always, in any new repo.
Then `gh repo create jvalol/<name> --public --source=. --push`, from inside
`games/<name>`.

## Registering it

Three files, in two other repos. `./check-listings` catches all three, so run it
rather than trusting the list.

1. `blitzkit-project/README.md`. The bullet in the game list **and** the
   `git clone` line under "Getting set up". check-listings only reads the first
   of those, so the clone line is on you.
2. `blitzkit/README.md`. The bullet under "Games built on it". The ones after
   marble carry an ordinal. The wording and the number are Jake's.
3. `blitzkit/site/content/docs/games.md`. A `## <name>` section. Descriptions
   open with "In which", which is deliberate. Then the GitHub link, then the
   screenshot:
   `![alt](https://raw.githubusercontent.com/jvalol/<name>/main/media/screenshot.png)`.
   The link is to raw.githubusercontent, so the image only appears once the game
   repo is pushed. Alt text describes what is in the picture, the way the other
   entries do, not what the game is. check-listings greps this file for that
   exact path, so the image is not optional.

`blitzkit-project/.gitignore` denies by default and names what it keeps. A game
is its own repo, so it needs nothing there. A new file at the project root does.

## Before it is done

Run `./check-all` from the project root. It runs tests, clippy and fmt for every
crate, then copy markers, spec citations, listings, and the tunnel/slider pair.

Every user-facing string drafted here carries `[COPY - Jake]` until Jake rewrites
it and strikes the marker himself. check-all reads `$repo/src` as well as the
readmes, so a marker in a string the game draws will hold the gate.
