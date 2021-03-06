use simple_logger::SimpleLogger;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use avfoundation::prelude::*;

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_min_inner_size(Some(LogicalSize::new(400.0, 200.0)));
    window.set_max_inner_size(Some(LogicalSize::new(800.0, 400.0)));

    let manager = AVAudioUnitComponentManager::shared();
    // let components = manager.components_passing_test(|unit| (true, ShouldStop::Continue));
    let components = manager.components_passing_test(|unit| {
        if unit.name().contains("Sylenth") {
            (true, ShouldStop::Stop)
        } else {
            (false, ShouldStop::Continue)
        }
    });

    let desc = components.first().unwrap().audio_component_description();

    // let engine = AVAudioEngine::new();
    // println!("start{:?}", engine.start());

    // println!("{:?}", components.first());

    // let midi = AVAudioUnitMIDIInstrument::new_with_audio_component_description(desc);

    // let (tx, rx) = std::sync::mpsc::channel();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut loaded_vc = false;

    // let tx2 = tx.clone();
    // let unit =
    // AVAudioUnit::new_with_component_description_fn(desc, Default::default(),move |unit| {

    // let _ = tx2.send(1);
    // tx2.send();
    // });
    let unit1 = AVAudioUnit::new_with_component_description_tx(desc, Default::default(), &tx);
    use avfoundation_sys::AUAudioUnitExt;
    // let unit = AVAudioUnitMIDIInstrument::new_with_audio_component_description(desc);

    // unit.au_audio_unit().request_view_controller_async(move |vc| {
    //     // let z = tx.send(avfoundation::AVFoundationEvent::RequestViewController(vc));
    //     println!("request view controller");
    // });

    // let mut v = vec![];
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        // println!("{:?}", event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: winit::event::ElementState::Released,
                        ..
                    },
                ..
            } => {}
            Event::MainEventsCleared => {
                use avfoundation::AVFoundationEvent;
                // for e in rx.try_recv() {
                //     let tx1 = tx.clone();
                //     unit.au_audio_unit().request_view_controller_async(move |controller| {
                //         println!("vc loaded");
                //         // tx1.send(10);
                //         // let _ = tx1.send(controller.to_owned());
                //     });
                // }
                // if !loaded_vc {
                for e in rx.try_recv() {
                    match e {
                        AVFoundationEvent::AVAudioUnitHandler(unit) => match unit {
                            Ok(unit) => {
                                println!("loaded audiounit");
                                let tx = tx.clone();
                                unit.au_audio_unit().request_view_controller_async(move |controller| {
                                    println!("request");
                                    let _ = tx.send(avfoundation::AVFoundationEvent::RequestViewController(Some(controller.to_owned())));
                                });
                            }
                            Err(e) => {
                                panic!("error {:?}", e);
                            }
                        },
                        AVFoundationEvent::RequestViewController(vc) => {
                            println!("loaded vc");
                            loaded_vc = true;
                            let vc = vc.unwrap();
                            // v.push(vc);
                            // let vc = v.last().unwrap();
                            window.window_with_content_view_controller(unsafe {
                                std::mem::transmute(vc)
                            });
                            // println!("vc {:?}", vc);
                        }
                    }
                }
                // }
            }
            _ => (),
        }
    });
}
