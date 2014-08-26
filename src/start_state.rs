use std::rc::Rc;
use std::cell::RefCell;

use rsfml;
use rsfml::window::event::{Closed, Resized, KeyPressed, NoEvent};
use rsfml::system::vector2::{ToVec, Vector2f, Vector2i};

use game;
use edit_state;

pub struct StartState {
    view: Rc<RefCell<rsfml::graphics::View>>
}

impl StartState {
    pub fn new(game: &game::Game) -> Option<StartState> {
        let size = game.window.get_size().to_vector2f();
        let center = size.mul(&0.5f32);

        let view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        Some(StartState {
            view: Rc::new(RefCell::new(view)),
        })
    }

    fn load_game(&self, game: &mut game::Game) {
        let state = edit_state::EditState::new(game).expect("could not load game");
        game.push_state(box state as Box<game::GameState>);
    }
}

impl game::GameState for StartState {
    fn draw(&mut self, dt: f32, game: &mut game::Game) {
        game.window.set_view(self.view.clone());
        game.window.clear(&rsfml::graphics::Color::black());
        game.window.draw(&game.background);
    }

    fn update(&mut self, dt: f32) {

    }

    fn handle_input(&mut self, game: &mut game::Game) {
        loop {
            match game.window.poll_event() {
                Closed => game.window.close(),
                Resized {width, height} => {
                    self.view.borrow_mut().set_size(&Vector2f::new(width as f32, height as f32));
                    let background_size = game.background.get_texture().unwrap().borrow().get_size();
                    game.background.set_position(&game.window.map_pixel_to_coords_current_view(&Vector2i::new(0, 0)));
                    game.background.set_scale(&Vector2f::new(width as f32 / background_size.x as f32, height as f32 / background_size.y as f32));
                },
                KeyPressed {code: rsfml::window::keyboard::Escape, ..} => game.window.close(),
                KeyPressed {code: rsfml::window::keyboard::Space, ..} => self.load_game(game),
                NoEvent => break,
                _ => {}
            }
        }
    }
}