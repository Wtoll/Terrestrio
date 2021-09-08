use crate::CustomEvent;

extern crate tokio;
use tokio::sync::mpsc;

extern crate winit;
use winit::event::WindowEvent;
use winit::window::WindowAttributes;
pub struct Window {
    winit: winit::window::Window,
    event_receiver: mpsc::UnboundedReceiver<WindowEvent<'static>>
}

impl Window {
    pub async fn new(attributes: WindowAttributes) -> Self {
        let (winit, event_receiver) = {
            let (send, receive) = tokio::sync::oneshot::channel();
            crate::send_event(CustomEvent::ConstructWindow {
                response: send,
                attributes
            });
            receive.await.unwrap()
        };

        Window {
            winit,
            event_receiver
        }
    }

    pub async fn poll_events(&mut self) -> WindowEvent<'static> {
        self.event_receiver.recv().await.unwrap()
    }
}