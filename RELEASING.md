# Releasing the engine

For whoever is cutting a release. Nothing here is needed to build on blitzkit:
`cargo add blitzkit` and you have it.

1. Bump `version` in `blitzkit/Cargo.toml`.
2. `./check-all`.
3. Commit the bump, then `cargo publish` from `blitzkit`, then tag it. That
   order matters both ways. Publishing before the commit puts a version on
   crates.io that corresponds to nothing, which 0.8.3 did for a few minutes.
   Tagging before the publish leaves a tag behind if the upload fails.
4. Run each game once so its lockfile picks up the new version, then commit the
   lockfiles. The override resolves blitzkit to the checkout, so a lockfile in
   here records whatever is on disk rather than what crates.io holds, and it
   changes on every release whether or not anyone commits it.
5. `./refresh-screenshots`, look at every picture it wrote, and commit them.
   Six of the thirteen were behind their own game before this existed, and a
   date check cannot tell you which: most of the commits since a shot was taken
   change only comments, and the arcade's was a release out of date while its
   dates looked fine.

The games' manifests name the minor version and need no edit unless it moves.

This page exists because none of it was written down. 0.8.3 went out improvised,
published with `--allow-dirty` ahead of its own commit, and sat on crates.io
corresponding to nothing until the bump was committed.

## Screenshots

A repo joins step 5 by taking a `--screenshot` argument that puts it in the
state worth photographing and holds it there. `refresh-screenshots` finds those
repos by reading their source, the way `check-all` reads the disk for its list,
so there is nothing to add here when one more is converted. Every game and the
arcade take it; a repo that does not is skipped and said so.

It stages rather than photographing whatever comes up, because a game's opening
frame rarely shows what the game is about. pong serves itself, carom takes a
shot and lets it settle, lantern puts a candle down and steps back to look at
it. Why each one stages what it does is in that game's own source.

Three things it has to do, each of which was found by it not being done. A game
photographed from behind the terminal never has focus, so one that pauses on
losing focus photographs its own pause screen. The staged state has to be held,
or it runs on while the window opens. And the staging has to happen where the
game can do it: pong lays itself out relative to the window, so its pose waits
for the first frame rather than running before `start`.

macOS only. It photographs a real window, because nothing in the engine renders
to a file.
