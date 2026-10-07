---
name: blitzkit-furnish-a-room
description: Furnish a room in the arcade, inside ~/Developer/blitzkit-project/arcade. Covers what makes a room stop looking bland, the faults that have been made in every room so far, and which of them a test can catch. Use when adding a room, filling one, or fixing one Jake has called bland, flat, or placeholder-looking.
---

# Furnishing a room in the arcade

The hall, the nook, the cellar and the baths were each built the same way, and
each of them went wrong the same way first. This is that list. None of it is
about taste: every item here is something Jake read off a screenshot and sent
back, more than once.

## The shape of the work

Write the spec first, then the geometry, then the drawing, then run it. Jake
looks at a screenshot and sends it back. Expect four or five rounds on a room
and do not treat any of them as a failure: the first build of a room is a
blocking model and he knows it.

Run `cargo run --release` from the repo, not from the project root, which has no
`Cargo.toml`. Lead every command with a `cd` to the real checkout.

## It is twice as small as you think

Every room has been too small on the first build and he has said so every time.
The nook went from 3.0 by 8.0 to 6.4 by 14.0. The baths went 11 by 9, then 13 by
10, then 18 by 11, and only the last one had walkways.

A body is 0.9 across. Anywhere somebody has to stand or pass needs about 1.05
clear of everything, and a gap of 0.6 is a gap you can see through and not get
to. Work out where the walkways are before placing anything, and leave the
furniture to fit them rather than the other way about.

## Boxes are placeholders and he will say so

Anything round, turned, thrown or carved comes off a wheel. `cellar::turned` is
public and takes a profile: barrels, bottles, urns, sconce bowls, pocket nets
and a fountain's basin are all built with it. A box stands in for exactly as
long as it takes to write the profile.

Watch three things with turned meshes:

- Too few steps and it collapses to nothing. `peg_mesh` with one step drew two
  points and no skin, which is why the candles, the clock dial and the fire's
  coals were all invisible at once.
- `lidded` spends the first and last tenth closing to a point. Right for a
  bottle, wrong for anything open: the fountain poured a pencil because its
  stream was the candles' mesh.
- Open at the top means two sided, or you see through the far wall of it.

Flat panels want the same treatment. One box is a two by four, which is what the
pool table's rails were. Three boards, a cap that oversails and a bead under it,
and the shadow lines do the rest.

## A room is a floor, walls, a ceiling and fittings

A plastered box with the right things in it still reads as a corridor. Each of
the four rooms needed all of:

- A floor you can tell from the walls, with a border or a pattern laid into it.
- Something on the walls between skirting and cornice: panelling in the cellar,
  tile and piers in the baths. A long wall with nothing breaking it is one slab.
- A ceiling that is not grey. The cellar's took coffers and beams.
- Fittings that have a reason: a sign, a seat, an urn, a fountain, a clock.

Finish a shared wall on both sides. The baths borrowed the cellar's wall and
had a mahogany wall down one side of a tiled room until somebody looked.

## Every lamp sits in a fitting

A light with no fitting is a bright patch on a wall and no reason for it. The
cellar was told this once and the baths did it again, worse: the lamps ran along
the ceiling and the sconces stood on a wall, so the ceiling glowed at nothing
and every sconce was dark.

Derive the lamps from the fittings. `spa::lamps` is `spa::sconces` lifted a
little, so there is no lamp that is not in a fitting and nobody has to remember.

## Two lists of one thing drift

This is the fault under most of the others. A sconce in a doorway, a grey patch
over the books, a stair foot written out nine times, the lamps and the sconces.
Where two pieces of code say where something is, they will come apart.

Name the number once and call it. `cellar::stair_foot`, `spa::doorway`,
`room::INSIDE`, `cellar::rails`: each exists because two places were saying the
same thing differently.

## Two surfaces in one plane fight

Four times now: a grey patch over the books, stripes round the pool, a
chequerboard down the wall of the baths, and the floor's border sunk into the
floor. Build behind a wall from its far face, not its near one. Stand a sign
clear of the panelling rather than clear of the stone behind it. Lay a border on
the floor, not in it.

Do not write the general rule. "No two boxes may overlap" is false and was tried:
a pier is deeper than the dado it breaks, a flight of steps nests, and a floor
slab sits on a basin wall's top. A face inside another solid is hidden rather
than fighting. Only two coplanar faces that are both on show actually fight, and
a list of boxes cannot say which faces are exposed. Test the specific
relationship that broke.

## Walk it, do not measure it

The pool's steps had four legal rises and were one solid block, because the
treads nested. Every rise passed. You could not get out.

So the tests walk. `spa::tests::walked` drives a body your size with the game's
own `walk` from a start to a target and says where it ended up. Use it for
getting in, getting out, and getting round. Start from several places, because
one path out is not a way out, and drop the start in from above rather than
placing it, because a point on the floor can be inside the furniture.

Include the neighbouring room's colliders. A walk that only knows this room's
boxes walks through the wall it shares.

## What the tests cannot see

Colour, mood, scale and whether a thing reads as what it is. Those come back
from the window and from Jake. Everything else is arithmetic and belongs in a
test: where a thing is, whether you fit, whether two things share a plane,
whether a mesh has a surface at all.

## The things that are easy to get backwards

- Colour is an unclamped multiplier. Past one it glows with no light on it.
  Neon is 3.2.
- Shininess is a power, so a bigger number is a smaller highlight. `DULL` is
  900 and is a pinpoint.
- A triangle wound away from you is culled and drawn nowhere. Winding and
  normals are decided by the same choice and are not the same thing.
- A cube's texture runs nought to one on every face however big the face is, so
  a tiling texture on a cube is one enormous tile. `room::tiled_plane` gives a
  quad its own counts.
- Edition 2018: `array.into_iter()` yields references. Use `.iter().copied()`.

## Before it is done

Run `cargo fmt`, then `cargo test`, then `cargo clippy --all-targets -- -D
warnings`, then `./check-all` from the project root. Cap the jobs: a full gate
builds sixteen crates and has filled this machine's memory more than once.

Edit by anchored replacement and verify afterwards. `cargo fmt` reflows what you
just wrote, so the next edit's anchor matches nothing and says nothing: three
times this session a change was reported as landed when the file was untouched.
Grep for the new text before saying it is in.

Every user facing string carries `[COPY - Jake]` until he strikes it himself.
