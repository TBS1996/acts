pub mod editpage;
pub mod picker;
pub mod treeview;

pub trait Page {
    fn refresh(&mut self);
}
