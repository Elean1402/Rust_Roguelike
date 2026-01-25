use tcod::colors::*;
use tcod::console::*;

// actual size of window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20; //20 frames per sec maximum

struct Tcod {
    root: Root,
}

struct Player {
    x: i32,
    y: i32,
}

fn handle_keys(tcod: &mut Tcod, player: &mut Player) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true, // exit game
        // movement keys
        Key { code: Up, .. } => player.y -= 1,
        Key { code: Down, .. } => player.y += 1,
        Key { code: Left, .. } => player.x -= 1,
        Key { code: Right, .. } => player.x += 1,

        _ => {}
    }
    false
}

fn main() {
    let mut tcod = Tcod {
        root:Root::initializer()
            .font("arial10x10.png", FontLayout::Tcod)
            .font_type(FontType::Greyscale)
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("Rust/libtcod tutorial")
            .init() };

    tcod::system::set_fps(LIMIT_FPS);

    let mut player = Player {
        x: SCREEN_WIDTH / 2,
        y: SCREEN_HEIGHT / 2,
    };

    while !tcod.root.window_closed() {
        tcod.root.set_default_foreground(WHITE);
        tcod.root.clear();
        tcod.root.put_char(player.x, player.y, '@', BackgroundFlag::None);
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
        // handle keys and exit game if needed
        let exit = handle_keys(&mut tcod, &mut player);
        if exit {
            break;
        }
    }
}
