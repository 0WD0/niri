use super::*;

fn grab_popup(f: &mut Fixture) {
    f.add_output(1, (1920, 1080));

    let id = f.add_client();
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);

    let window = f.client(id).window(&surface);
    window.attach_new_buffer();
    window.ack_last_and_commit();
    let parent = window.xdg_surface.clone();
    f.double_roundtrip(id);

    f.client(id).create_popup_grab(&parent, 1);
    f.roundtrip(id);
}

#[test]
fn popup_grab_without_touch_device() {
    let mut f = Fixture::new();
    assert!(f.niri().seat.get_touch().is_none());

    grab_popup(&mut f);

    assert!(f.niri().popup_grab.is_some());
}

#[test]
fn popup_grab_with_touch_device_grabs_touch() {
    let mut f = Fixture::new();
    let touch = f.niri().seat.add_touch();

    grab_popup(&mut f);

    assert!(touch.is_grabbed());
}

#[test]
fn popup_grab_before_touch_device_grabs_hotplugged_touch() {
    let mut f = Fixture::new();
    grab_popup(&mut f);

    f.niri_state().ensure_touch_handle();

    assert!(f.niri().seat.get_touch().unwrap().is_grabbed());
}
