pub trait Databasable {
    fn get_id(&self) -> Option<String>;
    fn set_id(&mut self, id: Option<String>);
}
