use polling::Event;

pub trait EventHandler {
    fn id(&self) -> usize;
    fn name(&self) -> String;
    fn poll(&mut self);
    fn event(&mut self, event: &Event);
    fn matches(&self, event: &Event) -> bool;
}
