---
name: blitzkit-screenshot
description: What a blitzkit picture has to be and where it has to be mentioned, inside ~/Developer/blitzkit-project. Covers the size, the paths, the alt text and the listing check. Use when a game repo needs media/screenshot.png, an example needs a picture, or check-listings is failing for a missing image. Take the picture with the mac-window-screenshot skill, which is the part that is the same everywhere.
---

# A picture for a blitzkit repo

Taking it is the `mac-window-screenshot` skill: finding the window, shooting it,
staging the frame so it shows something, and proving the staging did not
survive. Read that first. This is only what blitzkit wants of the result.

## The size and the chrome

Twelve hundred wide, because every other picture in these repos is:

```
sips --resampleWidth 1200 /tmp/raw.png --out media/screenshot.png
```

The window chrome stays in. The title bar is in every existing picture and one
without it looks like a different project.

## Where it goes and what has to mention it

`./check-listings` greps for the exact path and names the file that is missing
it, so run it rather than working from this list.

**A game** keeps `media/screenshot.png` in its own repo. The docs site names it
from `blitzkit/site/content/docs/games.md` as a raw.githubusercontent link, so
the image is broken until that game repo is pushed. That is expected and is not
something to chase.

**An example** keeps `blitzkit/media/<name>.png`, and three files name it: the
root readme, the engine readme, and
`blitzkit/site/content/docs/examples.md`. The site uses `/media/...` and the
engine readme uses the raw link.

## The alt text

Say what is in the picture, not what the thing is. "A rectangular pool of blue
water in a grey basin, waves running across the surface, a pale ball floating at
one end and two dark ones under the water" rather than "the ripple example".

## What a blitzkit frame has to show

An opening view is almost always the wrong frame here, because spec 0003 keeps a
cabinet's screen dark until somebody is within reach of it: from the doorway the
hall is a dark room with nothing answering. The arcade carries `--screenshot`
for this, which stands the camera a step back from a lit cabinet.

`ripple` needed a cork already floating, a granite ball already on the bottom, a
third caught mid splash, a push an order of magnitude bigger than the game makes,
the camera dropped nearer the water, and sixteen frames of settling rather than
forty, because by forty the rings had gone.
