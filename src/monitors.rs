use gettextrs::gettext;
use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle
};

#[derive(Default)]
pub struct AvailableMonitors {
    pub available_monitors: Vec<String>
}

impl Dispatch<wl_registry::WlRegistry, ()> for AvailableMonitors {
    fn event(
        _: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
	    if interface == "wl_output" {
		proxy.bind::<wl_output::WlOutput, _, _>(name, version.min(4), qhandle, ());
	    }
	}
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AvailableMonitors {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
	if let wl_output::Event::Name { name } = event {
	    state.available_monitors.push(name);
	}
    }
}

impl AvailableMonitors {
    pub fn get_monitors() -> anyhow::Result<Self> {
	let conn = Connection::connect_to_env()?;
	let mut event_queue = conn.new_event_queue::<AvailableMonitors>();
	let qh = event_queue.handle();
	conn.display().get_registry(&qh, ());
	let mut available_monitors = AvailableMonitors::default();
	event_queue.roundtrip(&mut available_monitors)?;
	event_queue.roundtrip(&mut available_monitors)?;
	available_monitors.available_monitors.push(gettext("All"));
	available_monitors.available_monitors.sort();
	Ok(available_monitors)
    }
}
