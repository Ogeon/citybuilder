use std::rc::Rc;
use std::cell::RefCell;

use rsfml;
use rsfml::window::event::{Closed, Resized, KeyPressed};
use rsfml::system::vector2::{ToVec, Vector2f, Vector2i};
use game;

pub struct EditState {
    game_view: Rc<RefCell<rsfml::graphics::View>>,
    gui_view: Rc<RefCell<rsfml::graphics::View>>
}

impl EditState {
    pub fn new(game: &game::Game) -> Option<EditState> {
        let size = game.window.get_size().to_vector2f();
        let center = size.mul(&0.5f32);

        let game_view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        let gui_view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        Some(EditState {
            game_view: Rc::new(RefCell::new(game_view)),
            gui_view: Rc::new(RefCell::new(gui_view)),
        })
    }
}

impl game::GameState for EditState {
    fn draw(&self, dt: f32, game: &mut game::Game) {
        game.window.clear(&rsfml::graphics::Color::black());
        game.window.draw(&game.background);
    }

    fn update(&mut self, dt: f32) {

    }

    fn handle_input(&mut self, game: &mut game::Game) {
        match game.window.poll_event() {
            Closed => game.window.close(),
            Resized {width, height} => {
                self.game_view.borrow_mut().set_size(&Vector2f::new(width as f32, height as f32));
                self.gui_view.borrow_mut().set_size(&Vector2f::new(width as f32, height as f32));
                let background_size = game.background.get_texture().unwrap().borrow().get_size();
                game.background.set_position(&game.window.map_pixel_to_coords_current_view(&Vector2i::new(0, 0)));
                game.background.set_scale(&Vector2f::new(width as f32 / background_size.x as f32, height as f32 / background_size.y as f32));
            },
            _ => {}
        }
    }
}