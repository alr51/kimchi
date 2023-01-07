use crate::KimchiApp;
use eframe::{egui::Ui, epaint::ColorImage};
use egui_extras::RetainedImage;
use ewebsock::{WsEvent, WsMessage};
use kimchi_messages::KimchiMessage;

pub struct Chat;

impl Chat {
    pub fn ui(app: &mut KimchiApp, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut app.msg_to_send);
            if ui.button("Send").clicked() {
                app.send_message();
            }
        });

        if let Some(receiver) = &mut app.ws_receiver {
            while let Some(ref event) = receiver.try_recv() {
                if let WsEvent::Message(WsMessage::Text(msg)) = event {
                    if let Some(message) = KimchiMessage::from_str(msg) {

                        match message {
                            KimchiMessage::Video { user:_, data } => {
                                let color_image = ColorImage::from_rgb([320,240],&data);
                                app.video = Some(RetainedImage::from_color_image("peer-webcam", color_image));
                            }
                            _ => app.messages.push(message)
                        }
                    }
                }
            }
        }

        for message in &app.messages {
            match message {
                KimchiMessage::Joined { user } => {
                    ui.heading(format!("{} Joined", user));
                }

                KimchiMessage::Public { user, message } => {
                    ui.heading(format!("{} {}", user, message));
                }

                KimchiMessage::Private {
                    user,
                    to_user,
                    message,
                } => {
                    ui.heading(format!("Private from {} to {} {}", user, to_user, message));
                }

                _ => ()
            }
        }

        // if let Some(video) = &app.videos.pop_front() {
        //     let color_image = ColorImage::from_rgba_unmultiplied([320,240],video);
        //
        //     let tex = ui
        //         .ctx()
        //         .load_texture("frame", color_image,TextureFilter::Linear); 
        //
        //     ui.image(&tex,tex.size_vec2());
        //
        // }

        if let Some(image) = &app.video {
            image.show(ui);
        }

        // ui.label(format!("Deque size: {}", app.videos.len()));

    }
}
