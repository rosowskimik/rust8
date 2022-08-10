mod args;
mod utils;

use anyhow::Result;
use game_loop::game_loop;

fn main() -> Result<()> {
    let args = args::parse_args();

    let (event_loop, window, game) = utils::setup(&args)?;

    game_loop(
        event_loop,
        window,
        game,
        *args
            .get_one("clock_speed")
            .expect("Clock speed should have default value"),
        0.1,
        move |g| {
            if g.game.rom_loaded && !g.game.paused {
                g.game.emulator.tick();
            }
        },
        move |g| {
            g.game.draw_screen();
        },
        move |g, event| {
            if g.game.handle_event(event) {
                g.exit();
            }
        },
    );
}
