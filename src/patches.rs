use rltk::{BEvent, Rltk, VirtualKeyCode, INPUT};

pub fn patch_mod_keys(ctx: &mut Rltk) {
    let mut input = INPUT.lock();

    ctx.shift = input.key_pressed_set().contains(&VirtualKeyCode::LShift)
        || input.key_pressed_set().contains(&VirtualKeyCode::RShift);

    ctx.control = input.key_pressed_set().contains(&VirtualKeyCode::LControl)
        || input.key_pressed_set().contains(&VirtualKeyCode::RControl);

    #[allow(clippy::single_match)]
    input.for_each_message(|event| match event {
        BEvent::CloseRequested => ctx.quitting = true,
        _ => (),
    });
}
