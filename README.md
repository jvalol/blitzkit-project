# blitzkit

A graphics engine built in Rust.

It's on crates.io and anyone, including you, can build with it. Add it to a
project and you get a window, a renderer and a physics step. The basics, basically. After that it's up to you. Be creative!

```
cargo add blitzkit
```

The engine lives in [blitzkit](blitzkit) and its API is on
[docs.rs](https://docs.rs/blitzkit). This repo is the whole project around it:
the engine itself, an arcade for demonstrations, and the games, which are each
their own repo.

Here are a few games demonstrating it.

- [blitzkit](blitzkit) — the engine, a wrapper around wgpu.
- [arcade](arcade) — the arcade. All games inside were built using this engine.
- [pong](games/pong) — the first game on it.
- [snake](games/snake) — the second.
- [tessera](games/tessera) — the third.
- [marble](games/marble) — the first one in 3D.
- [slider](games/slider) — a tunnel, and fourteen rings to thread.
- [starry](games/starry) — Van Gogh's Starry Night, sliced into tiles you slide.
- [lantern](games/lantern) — a dark maze, and two lamps to light it with.
- [securitysweep](games/securitysweep) — an open yard with four lights sweeping for security.
- [carom](games/carom) — thirteen marbles in a ring to shoot at.
- [poolhall](games/poolhall) — pool.
- [cairn](games/cairn) — a tower of blocks to take apart one at a time.
- [cascada](games/cascada) — dominoes to stand up and push over.

Enjoy the arcade with `cd arcade && cargo run --release`, or go
straight to one of the games.

You can run examples with e.g. `cd blitzkit && cargo run --release --example tunnel`
Or you can run a game with e.g. `cd games/cascada && cargo run --release`

Each game is its own repo. Many have corresponding examples in `blitzkit`.

## Getting set up

To build on the engine `cargo add blitzkit`.

Clone this repo.

```
git clone https://github.com/jvalol/blitzkit-project.git
cd blitzkit-project
git clone https://github.com/jvalol/blitzkit.git
git clone https://github.com/jvalol/arcade.git
mkdir -p games && cd games
git clone https://github.com/jvalol/pong.git
git clone https://github.com/jvalol/snake.git
git clone https://github.com/jvalol/tessera.git
git clone https://github.com/jvalol/marble.git
git clone https://github.com/jvalol/slider.git
git clone https://github.com/jvalol/starry.git
git clone https://github.com/jvalol/lantern.git
git clone https://github.com/jvalol/securitysweep.git
git clone https://github.com/jvalol/carom.git
git clone https://github.com/jvalol/poolhall.git
git clone https://github.com/jvalol/cairn.git
git clone https://github.com/jvalol/cascada.git
cd .. && ./check-all
```

Rust 1.87 or newer.

`./check-all` tests and lints every crate in dependency order, then runs
`./check-tunnel`.

Cutting a release is in [RELEASING.md](RELEASING.md).

## Demos

The engine carries its own, run from the `blitzkit` folder.

```
cargo run --release --example teapot
```

![The Utah teapot in white, spout to the left and handle to the right, lit from
above and casting a teapot shaped shadow](https://raw.githubusercontent.com/jvalol/blitzkit/main/media/teapot.png)

The Utah teapot, from the control points Martin Newell measured off a real one
in 1975. Thirty-two Bezier patches, each one a formula the engine tessellates
rather than a model file it loads.

![The same teapot in glass, its far wall, the underside of its lid and its
handle all showing through the near wall](https://raw.githubusercontent.com/jvalol/blitzkit/main/media/teapot-glass.png)

If you press T it turns translucent.

```
cargo run --release --example klein
```

![A Klein bottle drawn as a wire mesh, its neck curving over and back down into
its body, casting a lattice shadow on the floor](https://raw.githubusercontent.com/jvalol/blitzkit/main/media/klein.png)

A Klein bottle you can rotate. And you can explore mesh / translucent versions.

![The same bottle in glass, the neck visible carrying on down inside the body
after it passes through the wall](https://raw.githubusercontent.com/jvalol/blitzkit/main/media/klein-glass.png)

M gives you the solid surface and T makes it translucent.

```
cargo run --release --example tunnel
```

![Looking down a tunnel of dark and light checks receding to a vanishing point,
with a gold ring hanging off centre partway down it](https://raw.githubusercontent.com/jvalol/blitzkit/main/media/tunnel.png)

Fly down the inside of a surface. There are rings you can aim for while flying, but you don't have to. It's just a game after all.

`cubes` and `rolling`: lit textured geometry, and a ball with collision and
shadows. `stacking` is the physics holding itself up, a column and a pyramid of
spheres and a heap of blocks that stand instead of sinking, and `tower` is forty
blocks doing nothing at all, impressively.

Three recursive shapes, each made of copies of itself. Up and down change how
deep each one goes.

```
cargo run --release --example sierpinski
cargo run --release --example menger
cargo run --release --example hilbert
```

`sierpinski` is four copies of itself with the middle left out, `menger` twenty
with the middle drilled out of every face, and `hilbert` a line that fills a
cube, drawn as a tube.
