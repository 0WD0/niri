use ::input as libinput;
use smithay::backend::input;
use smithay::backend::winit::WinitVirtualDevice;
use smithay::output::Output;

use crate::niri::State;
use crate::protocols::virtual_pointer::VirtualPointer;

pub trait NiriInputBackend: input::InputBackend<Device = Self::NiriDevice> {
    type NiriDevice: NiriInputDevice;
}
impl<T: input::InputBackend> NiriInputBackend for T
where
    Self::Device: NiriInputDevice,
{
    type NiriDevice = Self::Device;
}

pub trait NiriInputDevice: input::Device {
    // FIXME: this should maybe be per-event, not per-device,
    // but it's not clear that this matters in practice?
    // it might be more obvious once we implement it for libinput
    fn output(&self, state: &State) -> Option<Output>;

    /// Whether this device's absolute coordinates are reported in an *unrotated* coordinate
    /// space, i.e. the device does not physically rotate together with the output panel.
    ///
    /// True for purely virtual `INPUT_PROP_DIRECT`/absolute devices such as the ones created by
    /// inputtino for Sunshine/Wolf. For these devices, applying the output's `transform` to the
    /// reported position would rotate the input one extra time relative to what the user sees.
    ///
    /// Real physical touchscreens are usually mounted on the panel and their sensor frame rotates
    /// together with the output's `transform`, so the default is `false`.
    fn is_unrotated_absolute_device(&self) -> bool {
        false
    }
}

impl NiriInputDevice for libinput::Device {
    fn output(&self, _state: &State) -> Option<Output> {
        // FIXME: Allow specifying the output per-device?
        None
    }

    fn is_unrotated_absolute_device(&self) -> bool {
        // Virtual absolute devices created via uinput by game-streaming hosts (Sunshine and Wolf,
        // both via the inputtino library) are not physically attached to any output panel, so
        // their X/Y axes are always reported in a "normal" frame regardless of the output's
        // transform. Detect them by name so that `compute_absolute_location` can skip the
        // output-transform step for them. Without this, on a `transform=90`/`270` portrait
        // output, touches from these devices would land 90 degrees rotated relative to what the
        // user sees on screen, even though the rendered video is correct.
        //
        // Names emitted by inputtino (see
        // `third-party/inputtino/src/uinput/{mouse,touchscreen}.cpp` in Sunshine, and the device
        // definitions in `src/platform/linux/input/inputtino_common.h`) include:
        //   - "Mouse passthrough" / "Mouse passthrough (absolute)"
        //   - "Touch passthrough"
        //   - "Pen passthrough"
        //   - "Wolf mouse virtual device (absolute)"
        //   - "Wolf touch screen virtual device"
        //   - "Wolf pen virtual device"
        // The Sunshine "Mouse passthrough" name is also used for the relative pointer node, but
        // relative motion is not routed through `compute_absolute_location` so matching it here
        // is harmless.
        let lower = libinput::Device::name(self).to_ascii_lowercase();
        lower.starts_with("wolf ")
            || lower.contains("virtual device")
            || lower.contains("passthrough")
    }
}

impl NiriInputDevice for WinitVirtualDevice {
    fn output(&self, _state: &State) -> Option<Output> {
        // FIXME: we should be returning the single output that the winit backend creates,
        // but for now, that will cause issues because the output is normally upside down,
        // so we apply Transform::Flipped180 to it and that would also cause
        // the cursor position to be flipped, which is not what we want.
        //
        // instead, we just return None and rely on the fact that it has only one output.
        // doing so causes the cursor to be placed in *global* output coordinates,
        // which are not flipped, and happen to be what we want.
        None
    }
}

impl NiriInputDevice for VirtualPointer {
    fn output(&self, _: &State) -> Option<Output> {
        self.output().cloned()
    }

    fn is_unrotated_absolute_device(&self) -> bool {
        // The virtual-pointer protocol speaks in compositor-logical coordinates and does not
        // carry any physical orientation, so the output transform must not be re-applied here.
        true
    }
}
