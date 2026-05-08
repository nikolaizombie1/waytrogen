use crate::locale::TRANSLATION;
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_output, wl_registry},
};

#[derive(Default)]
pub struct AvailableMonitors {
    pub available_monitors: Vec<String>,
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
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "wl_output"
        {
            proxy.bind::<wl_output::WlOutput, _, _>(name, version.min(4), qhandle, ());
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
        available_monitors.available_monitors.sort();
        available_monitors
            .available_monitors
            .insert(0, TRANSLATION.get_translation("All"));
        Ok(available_monitors)
    }
}
