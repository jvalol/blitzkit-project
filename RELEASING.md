# Releasing the engine

For whoever is cutting a release. Nothing here is needed to build on blitzkit:
`cargo add blitzkit` and you have it.

```
./release 0.13.0
```

That is the whole thing. It checks every repo is clean and level, bumps
`blitzkit/Cargo.toml`, bumps the games' manifests if the minor moved, runs
`./check-all`, commits the bump, asks you once before the upload, publishes,
tags, pushes, regenerates every screenshot and commits the lockfiles and
pictures that follow.

Look at the thirteen pictures afterwards. A staged shot can stage the wrong
thing and nothing but a person can see that.

## Why it is a script

It was five numbered steps and fifteen repos done by hand, and the order is not
obvious from the outside while mattering twice over.

**Publish after the commit.** 0.8.3 went out improvised, uploaded with
`--allow-dirty` ahead of its own commit, and sat on crates.io corresponding to
nothing until the bump was committed.

**Tag after the publish.** A tag left behind by an upload that failed is worse
than no tag at all.

**Push the games after the publish.** Their manifests name the minor, so the
moment that moves they are asking for a version that does not exist yet. Push
them first and a clone of any game cannot build until the upload lands.

**The lockfiles move whether or not anyone commits them.** The
`[patch.crates-io]` resolves blitzkit to the checkout, so a lockfile in here
records what is on disk rather than what crates.io holds. Leaving them is a
dozen dirty repos; committing them before the bump is a lockfile naming a
release that does not exist.

v0.9.6 and v0.10.0 both shipped untagged and were tagged by hand later, which is
the kind of thing a list of steps loses and a script does not.

## What it will not do

Publish without being asked. The upload cannot be taken back, so it stops and
waits for a yes. Everything before that point is committed and pushed nowhere,
so `git reset --hard` undoes it.

## Screenshots

A repo joins the regeneration by taking a `--screenshot` argument that puts it
in the state worth photographing and holds it there. `refresh-screenshots` finds
those repos by reading their source, the way `check-all` reads the disk for its
list, so there is nothing to add here when one more is converted. Every game and
the arcade take it; a repo that does not is skipped and said so.

It stages rather than photographing whatever comes up, because a game's opening
frame rarely shows what the game is about. pong serves itself, carom takes a
shot and lets it settle, lantern puts a candle down and steps back to look at
it. Why each one stages what it does is in that game's own source.

macOS only. It photographs a real window, because nothing in the engine renders
to a file.
