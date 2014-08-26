use std::rc::Rc;
use std::cell::RefCell;

use rsfml;
use rsfml::window::event::{
    Closed,
    Resized,
    MouseMoved,
    MouseWheelMoved,
    MouseButtonPressed,
    MouseButtonReleased,
    NoEvent
};
use rsfml::window::mouse;
use rsfml::system::vector2::{ToVec, Vector2f, Vector2i};
use game;

enum ActionState {
    Nothing,
    Panning(Vector2f)
}

pub struct EditState {
    game_view: Rc<RefCell<rsfml::graphics::View>>,
    gui_view: Rc<RefCell<rsfml::graphics::View>>,
    map: game::Map,
    action_state: ActionState,
    zoom_level: f32
}

impl EditState {
    pub fn new(game: &game::Game) -> Option<EditState> {
        let size = game.window.get_size().to_vector2f();
        let center = size.mul(&0.5f32);

        let gui_view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        let map = game::Map::new_generated(game.tile_size, &game.tile_atlas);

        let (width, height) = map.size();

        let center = Vector2f::new(
            (width * game.tile_size) as f32,
            (height * game.tile_size) as f32 * 0.5
        );

        let mut game_view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        Some(EditState {
            game_view: Rc::new(RefCell::new(game_view)),
            gui_view: Rc::new(RefCell::new(gui_view)),
            map: map,
            action_state: Nothing,
            zoom_level: 1.0
        })
    }
}

impl game::GameState for EditState {
    fn draw(&mut self, dt: f32, game: &mut game::Game) {
        game.window.clear(&rsfml::graphics::Color::black());
        game.window.set_view(self.gui_view.clone());
        game.window.draw(&game.background);

        game.window.set_view(self.game_view.clone());
        self.map.draw(&mut game.window, dt);
    }

    fn update(&mut self, dt: f32) {

    }

    fn handle_input(&mut self, game: &mut game::Game) {
        loop {
            match game.window.poll_event() {
                Closed => game.window.close(),
                Resized {width, height} => {
                    let size = Vector2f::new(width as f32, height as f32);
                    self.game_view.borrow_mut().set_size(&size);
                    self.game_view.borrow_mut().zoom(self.zoom_level);
                    self.gui_view.borrow_mut().set_size(&size);
                    let background_size = game.background.get_texture().unwrap().borrow().get_size();
                    game.background.set_position(&game.window.map_pixel_to_coords_current_view(&Vector2i::new(0, 0)));
                    game.background.set_scale(&Vector2f::new(width as f32 / background_size.x as f32, height as f32 / background_size.y as f32));
                },
                MouseMoved {x, y} => match self.action_state {
                    Panning(ref mut anchor) => {
                        let pos = Vector2f::new(anchor.x - x as f32, anchor.y - y as f32);
                        self.game_view.borrow_mut().move(&pos.mul(&self.zoom_level));
                        *anchor = Vector2f::new(x as f32, y as f32);
                    },
                    _ => {}
                },
                MouseButtonPressed {x, y, button: mouse::MouseMiddle} => match self.action_state {
                    Nothing => self.action_state = Panning(Vector2f::new(x as f32, y as f32)),
                    _ => {}
                },
                MouseButtonReleased {button: mouse::MouseMiddle, ..} => self.action_state = Nothing,
                MouseWheelMoved {delta, ..} if delta > 0 => {
                    self.game_view.borrow_mut().zoom(2.0);
                    self.zoom_level *= 2.0;
                },
                MouseWheelMoved {delta, ..} if delta < 0 => {
                    self.game_view.borrow_mut().zoom(0.5);
                    self.zoom_level *= 0.5;
                },
                NoEvent => break,
                _ => {}
            }
        }
    }
}