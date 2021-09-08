use std::collections::HashMap;

extern crate tokio;
use tokio::runtime;
use tokio::sync::mpsc;

extern crate winit;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopClosed;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowAttributes;
use winit::window::WindowBuilder;
use winit::window::WindowId;

pub mod window;

static mut GLOBAL_EVENT_LOOP_PROXY: Option<EventLoopProxy<CustomEvent>> = Option::None;

pub enum CustomEvent {
    ConstructWindow {
        response: tokio::sync::oneshot::Sender<(winit::window::Window, mpsc::UnboundedReceiver<WindowEvent<'static>>)>,
        attributes: WindowAttributes
    },
    Exit
}

fn main() {
    let tokio = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let event_loop = EventLoop::<CustomEvent>::with_user_event();
    unsafe { GLOBAL_EVENT_LOOP_PROXY = Option::Some(event_loop.create_proxy()) }
    let mut dispatch: HashMap<WindowId, mpsc::UnboundedSender<WindowEvent<'static>>> = HashMap::new();

    let _guard = tokio.enter();
    tokio.spawn(async_main());

    event_loop.run(move |event, event_loop_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent{ window_id, event: WindowEvent::Destroyed } => {
                dispatch.remove(&window_id);
            },
            Event::WindowEvent{ window_id, event } => {
                let to_window = dispatch.get(&window_id).unwrap().clone();
                let static_event = event.to_static().unwrap();
                tokio::spawn(async move {
                    to_window.send(static_event);
                });
            },
            Event::UserEvent(CustomEvent::ConstructWindow{ response, attributes }) => {
                let mut builder = WindowBuilder::new();
                builder.window = attributes;
                let window = builder.build(event_loop_target).unwrap();
                let (window_event_send, window_event_receive) = mpsc::unbounded_channel();
                dispatch.insert(window.id(), window_event_send);
                response.send((window, window_event_receive));
            },
            Event::UserEvent(CustomEvent::Exit) => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

async fn async_main() {
    
    let mut window = window::Window::new(WindowAttributes {
        ..Default::default()
    }).await;

    loop {
        match window.poll_events().await {
            WindowEvent::CloseRequested => {
                drop(window);
                break;
            }
            event => println!("{:?}", event)
        }
    }

    send_event(CustomEvent::Exit);
}

pub fn send_event(event: CustomEvent) -> Result<(), EventLoopClosed<CustomEvent>> {
    get_proxy().send_event(event)
}

fn get_proxy() -> EventLoopProxy<CustomEvent> {
    unsafe { (GLOBAL_EVENT_LOOP_PROXY.as_ref().unwrap()).clone() }
}