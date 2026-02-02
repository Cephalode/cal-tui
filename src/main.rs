mod calendar;
mod events;
mod views;
mod ui;
mod csv;

use std::io;
use ui::App;

fn main() -> io::Result<()> {
    let mut app = App::new();
    app.run()
}
