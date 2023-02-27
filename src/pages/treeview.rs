use crate::activity::Activity;
use crate::ActID;
use crate::Conn;
use crate::Message;
use iced::widget::{button, column, text_input};

use iced::{Alignment, Element, Renderer, Sandbox};
struct TreeView {
    activities: Vec<Activity>,
}
