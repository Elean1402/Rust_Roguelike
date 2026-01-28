use std::collections::HashMap;
use tcod::colors::*;
use tcod::console::*;
use std::cmp;
use std::cmp::PartialEq;
use rand::{random_range, Rng};

// actual size of window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 60; //20 frames per sec maximum
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

struct Tcod {
    root: Root,
    con: Offscreen,
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }
    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

// rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}
impl PartialEq for Rect {
    fn eq(&self, other: &Self) -> bool {
        (self.x1 == other.x2)
            && (self.x2 == other.x1)
            && (self.y1 == other.y2)
            && (self.y2 == other.y1)
    }
}
impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }
    pub fn intersects_with(&self, other: &Rect) -> bool {
        // returns true if this rectangle intersects with another one
        (self.x1 <= other.x2)
        && (self.x2 >= other.x1)
        && (self.y1 <= other.y2)
        && (self.y2 >= other.y1)
    }
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}
impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }
    // move by the given amount
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }
    // set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

struct Objects {
    player: Object,
    npcs: HashMap<String, Object>,
}
impl Objects {
    pub fn new(player: Object, npcs: HashMap<String, Object>) -> Self {
        Objects { player, npcs }
    }
    pub fn draw_all(&self, con: &mut dyn Console) {
        for npc in self.npcs.values() {
            npc.draw(con);
        }
        self.player.draw(con);
    }
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
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
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),
        _ => {}
    }
    false
}

fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &Objects) {
    // draw all objects in the list
    objects.draw_all(&mut tcod.con);

    // go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }

    blit(
        &tcod.con,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

fn create_tunnel(x1: i32, x2: i32, y: i32, y2: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) +1) {
        for y in cmp::min(y, y2)..(cmp::max(y, y2) +1) {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}



fn make_map(player: &mut Object) -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    for x in 0..MAX_ROOMS {
        let w = rand::random_range(ROOM_MIN_SIZE..ROOM_MAX_SIZE +1);
        let h = rand::random_range(ROOM_MIN_SIZE..ROOM_MAX_SIZE +1);
        let room = Rect::new(rand::random_range(0..MAP_WIDTH - w),
                              rand::random_range(0..MAP_HEIGHT - h), w, h);
        let failed = rooms.iter().any(|other| room.intersects_with(other));
        if !failed{
            create_room(room, &mut map);
            let (cen_x, cen_y) = room.center();
            if rooms.is_empty() {
                player.x = cen_x;
                player.y = cen_y;
            }
            rooms.push(room);
        }
    }

    map
}

fn main() {
    let mut tcod = Tcod {
        root: Root::initializer()
            .font("arial10x10.png", FontLayout::Tcod)
            .font_type(FontType::Greyscale)
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("Rust/libtcod tutorial")
            .init(),
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
    };

    tcod::system::set_fps(LIMIT_FPS);

    let player = Object::new(25, 23, '@', WHITE);
    let mut npcs = HashMap::new();
    npcs.insert(
        "bob".to_string(),
        Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW),
    );
    let mut objects = Objects::new(
        player,
        npcs,
    );
    objects.player.draw(&mut tcod.con);

    let game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(&mut objects.player),
    };

    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();
        render_all(&mut tcod, &game, &objects);
        tcod.root.flush();

        tcod.root.wait_for_keypress(true);
        // handle keys and exit game if needed
        let exit = handle_keys(&mut tcod, &game, &mut objects.player);
        if exit {
            break;
        }
    }
}
