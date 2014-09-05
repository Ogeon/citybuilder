use std::rc::Rc;
use std::cell::RefCell;

use rsfml;
use rsfml::window::event::{Closed, Resized, KeyPressed, MouseMoved, MouseButtonReleased, NoEvent};
use rsfml::window::mouse;
use rsfml::system::vector2::{ToVec, Vector2f, Vector2i};

use game;
use edit_state;
use gui;

pub struct StartState<'s> {
    view: Rc<RefCell<rsfml::graphics::View>>,
    menu: gui::Gui<'s, 'static, &'static str>
}

impl<'s> StartState<'s> {
    pub fn new(game: &game::Game) -> Option<StartState<'s>> {
        let size = game.window.get_size().to_vector2f();
        let center = size.mul(&0.5f32);

        let view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        let mut menu = gui::Gui::new(
            Vector2f::new(192.0, 32.0), 4, false,
            game.stylesheets.find(&"button").unwrap().clone(),
            vec![("New Game", "new_game")]
        );

        menu.transform.set_position(&center);
        menu.transform.set_origin(&Vector2f::new(96.0, 16.0));
        menu.show();

        Some(StartState {
            view: Rc::new(RefCell::new(view)),
            menu: menu
        })
    }

    fn load_game(&self, game: &mut game::Game) {
        let state = edit_state::EditState::new(game).expect("could not load game");
        game.push_state(box state as Box<game::GameState>);
    }
}

impl<'s> game::GameState for StartState<'s> {
    fn draw(&mut self, _dt: f32, game: &mut game::Game) {
        game.window.set_view(self.view.clone());
        game.window.clear(&rsfml::graphics::Color::black());
        game.window.draw(&game.background);
        game.window.draw(&self.menu);
    }

    fn update(&mut self, _dt: f32) {

    }

    fn handle_input(&mut self, game: &mut game::Game) {
        let mouse_pos = game.window.map_pixel_to_coords(&game.window.get_mouse_position(), self.view.borrow().deref());

        loop {
            match game.window.poll_event() {
                Closed => game.window.close(),
                Resized {width, height} => {
                    self.view.borrow_mut().set_size(&Vector2f::new(width as f32, height as f32));
                    let background_size = game.background.get_texture().unwrap().borrow().get_size();
                    game.background.set_position(&game.window.map_pixel_to_coords(&Vector2i::new(0, 0), self.view.borrow().deref()));
                    game.background.set_scale(&Vector2f::new(width as f32 / background_size.x as f32, height as f32 / background_size.y as f32));
                },
                KeyPressed {code: rsfml::window::keyboard::Escape, ..} => game.window.close(),
                MouseMoved {..} => {
                    let index = self.menu.get_entry(&mouse_pos);
                    self.menu.highlight(index);
                },
                MouseButtonReleased {button: mouse::MouseLeft, ..} => {
                    match self.menu.activate_at(&mouse_pos) {
                        Some(&"new_game") => self.load_game(game),
                        _ => {}
                    }
                },
                NoEvent => break,
                _ => {}
            }
        }
    }
}