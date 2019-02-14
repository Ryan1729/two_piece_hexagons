use web;

use game;

fn main() {
    let params = web::get_state_params();
    let state = game::EntireState::new(params);
    web::run(state);
}
