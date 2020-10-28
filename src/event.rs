use termion::event::Key;

pub enum Event {
    Input(Key),
    Redraw,
}
