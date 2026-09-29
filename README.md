# blitzkit

A graphics engine in Rust, and the games that prove it works.

- [blitzkit](blitzkit) — the engine, a wrapper around wgpu.
- [pong](pong) — the first game on it.
- [snake](snake) — the second.
- [tessera](tessera) — the third.
- [marble](marble) — the first one in 3D.
- [slider](slider) — a tunnel, and fourteen rings to thread.
- [starry](starry) — Van Gogh's Starry Night, sliced into tiles you slide.

Each is its own repo, and each depends on the published blitzkit the way anyone
else would. They live together here because `.cargo/config.toml` overrides that
with the engine checkout, which is what keeps the engine honest: every game is a
test of using it from outside, and a breaking change shows up before it ships.

`./check-all` tests and lints every crate in dependency order, then runs
`./check-tunnel`, which holds blitzkit's tunnel example and the slider game to
the same numbers. Those two share an idea rather than any code, and neither
repo can see the other, so that script is the only place the pair can be kept
honest. `.cargo/config.toml`
points them at one shared build directory.

## Releasing the engine

A release starts in `blitzkit` and finishes in the games, so it touches every
repo here.

1. Bump `version` in `blitzkit/Cargo.toml`.
2. `./check-all`.
3. Commit the bump, then `cargo publish` from `blitzkit`, then tag it. That
   order matters both ways. Publishing before the commit puts a version on
   crates.io that corresponds to nothing, which 0.8.3 did for a few minutes.
   Tagging before the publish leaves a tag behind if the upload fails.
4. Run each game once so its lockfile picks up the new version, then commit the
   five lockfiles. The override resolves blitzkit to the checkout, so a lockfile
   in here records whatever is on disk rather than what crates.io holds, and it
   changes on every release whether or not anyone commits it.

The games' manifests say `blitzkit = "0.8"` and need no edit unless the minor
version moves.

## Getting set up

Clone this repo, then clone the others inside it. The override names the engine
folder `blitzkit`, so that one matters. What this folder is called does not, and
outside it the games build against crates.io.

```
git clone git@github.com:jvalol/blitzkit-project.git
cd blitzkit-project
git clone git@github.com:jvalol/blitzkit.git
git clone git@github.com:jvalol/pong.git
git clone git@github.com:jvalol/snake.git
git clone git@github.com:jvalol/tessera.git
git clone git@github.com:jvalol/marble.git
git clone git@github.com:jvalol/slider.git
git clone git@github.com:jvalol/starry.git
./check-all
```

Rust 1.87 or newer, which is wgpu's minimum. Each game runs with `cargo run`
from its own folder.

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

`cubes` and `rolling` are the other two: lit textured geometry, and a ball with
collision and shadows.

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
