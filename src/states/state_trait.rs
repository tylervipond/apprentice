use rltk::Rltk;
use specs::World;
pub trait State {
    fn build(&self, world: &World, ctx: &Rltk);
    fn draw(&self, world: &World, ctx: &mut Rltk);
    fn get_event();
}
