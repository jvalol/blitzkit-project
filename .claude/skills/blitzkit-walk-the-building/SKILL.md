---
name: blitzkit-walk-the-building
description: Walk a blitzkit game on its own and look for the faults this project keeps making, inside ~/Developer/blitzkit-project. Covers driving the camera without a hand on the keyboard, the seven faults worth hunting, which of them a test can catch instead, and how to hand Jake a position rather than a guess. Use before showing him a room, after changing geometry or lighting, or when he reports something he can see and you cannot.
---

# Walking the building and looking for faults

Jake finds these by playing. Every one in this file was found that way and
most of them took several rounds, because the loop is slow: he plays, he sends
a picture, and the picture is of somewhere you then have to find.

This is the other loop. Drive the camera yourself, look at what comes back,
and only hand him something when you have run out of what a machine can see.

## Drive it, do not pose it

Posing the camera at a guessed position is the thing that wastes the most
time. Six guesses at where to point it found nothing on 2026-10-08; the sky
dome turned out to be inside the building and one position from Jake found it
immediately.

So: make the game walk, and make it say where it is.

```rust
// in update, behind an env var, never committed
if std::env::var("WALK").is_ok() {
    self.yaw += dt * 0.37;
    wish = self.forward();
}
if std::env::var("WHERE").is_ok() {
    self.said += dt;
    if self.said >= 0.5 {
        self.said = 0.0;
        eprintln!("WHERE at {:.2} {:.2} {:.2}  yaw {:.3}  pitch {:.3}",
            self.at.x, self.at.y, self.at.z, self.yaw, self.pitch);
    }
}
```

`WALK` drives it round a room on its own while you watch RSS or read the log.
`WHERE` is the one to reach for the moment Jake says he can see something: ask
him to stand in it, read the last line of `/tmp/arcade.log`, and put the
camera exactly there. One round trip instead of six.

Take the pictures with the `mac-window-screenshot` skill. Two traps:

- **Kill every other instance first.** Jake's own arcade and your staged one
  both have windows called `arcade`, and `swift winid.swift | head -1` will
  happily photograph his. Two wasted rounds on 2026-10-08 were this.
- **Restore `POSED_AT`, `POSED_YAW` and `POSED_PITCH` and prove it**, with
  `git diff | grep -c POSED` returning 0 before you say you are done.

## What to look for

Walk each room, stop in the middle, and look up, down and at each wall in
turn. Up is the one that gets skipped and is where three of these live.

**A surface stretched over something too big for it.** A cube's texture runs
nought to one on every face however big the face is, so one board's grain on a
four metre wall is that board at four metres: blocks you can count and a
cathedral figure the size of your head. Look for enormous soft scallops and
visible texels. The fix is tiled quads laid on the face, not a bigger texture.

**Two exposed faces in one plane.** Speckle that changes as you move, in a
band or a patch. This building has hit it seven times: stripes round the pool,
a chequerboard down a wall, a border sunk into a floor, a sign on panelling,
the sky in patches, a floor over a garden, the underside of a neon can. Two
boxes overlapping is fine and hidden; it is only two faces *both on show* that
fight.

**Something one room is drawing inside another.** Look up in every room. The
sky dome was nineteen across against a garden of sixteen, so it reached over
the hall and the nook, and sprung below their ceilings its rim ran through
them at head height. A pale blade hanging through a ceiling is this.

**A gap where two rooms meet.** Where a wall is borrowed from a shorter room,
the strip above it is open unless somebody topped it. Done twice here and
missed once each time. Sweep the boundary at the height between the two
ceilings rather than checking the corners; the gap is in the middle of the
wall.

**A hole or a ledge where two floors meet.** What two floors owe each other is
ground with no hole to drop through and no ledge to stumble up. A room's roof
standing proud of the next room's floor is the one that hides best: you walk
and your feet rise, and nothing says so. Hunt it by printing the solid boxes
that overlap a room's footprint and comparing tops, not by looking. Check the
join for daylight between the slabs, and check the doorway has floor under
the whole of its width.

There used to be a third of these and there is not any more. A vertical face
of a floor slab you crossed while standing on it stopped you dead, `walk`
reported it as nothing at all, and it was found three times in one day. Spec
0007's step-over ended that, and on 2026-10-28 it would not reproduce
anywhere: not at any seam in the arcade with the walls taken away, not on a
pair of slabs built to meet edge to edge, and not with the climb budget
spent. Do not go hunting it.

**Anything drawn over the help text.** The text has no background of its own
in most of these games. Walk up to the brightest thing in the room and read
the top left corner.

**Memory, when the window is not being looked at.** Start the game, hide its
window, and watch RSS for twenty seconds. A frame that does not draw still
has to submit, or every `write_buffer` since the last submit is kept: the
arcade grew by half a gigabyte a second that way and took the machine down
four times over six days before anybody caught it, because it only happened
while nobody was watching. Always run this with a watchdog that kills at a
couple of gigabytes.

```bash
pid=$(pgrep -n -f 'target/debug/arcade')
osascript -e 'tell application "System Events" to set visible of (first process whose unix id is '"$pid"') to false'
for i in $(seq 1 12); do
  mb=$(( $(ps -o rss= -p $pid) / 1024 )); echo "${i}0s ${mb} MB"
  [ "$mb" -gt 2000 ] && { echo RUNAWAY; kill -9 $pid; break; }; sleep 5
done
```

## Prefer a test to an eye

Most of the list above is arithmetic and belongs in a test, where it holds
for ever instead of until the next room. Each of these exists in the arcade
and is worth copying rather than reinventing:

| Fault | The test that holds it |
| --- | --- |
| A room drawn over another | `garden::tests::nothing_of_the_hall_is_laid_over_the_garden` |
| The sky indoors | `garden::tests::no_part_of_the_sky_is_inside_the_building` |
| A gap above a borrowed wall | `garden::tests::the_garden_is_closed_all_the_way_up` |
| A hole or a ledge where floors meet | `behind::tests::the_floors_leave_no_hole_and_no_ledge`, and `garden::tests::the_garden_sits_on_the_roof_of_the_baths` for a room laid over a room |
| A frame that draws nothing | `renderer::asked::a_frame_that_draws_nothing_still_flushes` |
| Help unreadable over the room | `aim::tests::every_line_of_text_sits_on_its_plate` |

**Give every one a negative control.** Break the thing it guards, watch it
fail, put it back. A test written after a fix passes whether or not it is
testing anything, and at least one on this list did.

## What only Jake can judge

Colour, mood, scale, whether a thing reads as what it is, and whether a room
is worth standing in. Do not walk the building looking for those. Take one
good picture of the thing he asked for and let him say.

And when you cannot find what he is looking at, say so after two or three
honest attempts rather than after six. The `WHERE` log costs him five seconds
and costs you nothing.
