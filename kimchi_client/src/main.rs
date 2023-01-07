use std::collections::VecDeque;

use clap::Parser;
use eframe::{egui, epaint::ColorImage};
use egui_extras::RetainedImage;
use ewebsock::{WsMessage, WsReceiver, WsSender};
use kimchi_messages::KimchiMessage;
use nokhwa::{native_api_backend,query, utils::{RequestedFormat, RequestedFormatType, Resolution}, Camera, pixel_format::RgbFormat};

mod chat;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("ws://localhost:4000"))]
    url: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let options = eframe::NativeOptions::default();
   
    eframe::run_native(
        "Kimchi client",
        options,
        Box::new(|_cc| Box::new(KimchiApp::new(args.url))),
    );
}

#[derive(Default)]
pub struct KimchiApp {
    pub server_url: String,
    pub user_to_set: String,
    pub current_user: String,
    pub msg_to_send: String,
    pub messages: Vec<KimchiMessage>,
    pub ws_sender: Option<WsSender>,
    pub ws_receiver: Option<WsReceiver>,
    pub camera: Option<Camera>,
    pub videos :VecDeque<Vec<u8>>,
    pub video: Option<RetainedImage>
}

impl KimchiApp {
    fn new(server_url: String) -> Self {
        let mut app = KimchiApp::default();
        app.server_url = server_url;

        // Init Webcam
        let backend = native_api_backend().unwrap();
        let devices = query(backend).unwrap();

        let cam_info = devices.last().clone().expect("Camera Info expected");
        let index = cam_info.index();
        let resolution = Resolution::new(320, 240);

        if let Ok(mut camera) = Camera::new(index.to_owned(), RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(resolution))){
            _ = camera.open_stream();
            app.camera = Some(camera);
        } else {
            app.camera = None;
        }
        
        app

       
    }

    fn send_message(&mut self) {
        let message = KimchiMessage::Public {
            user: self.current_user.clone(),
            message: self.msg_to_send.clone(),
        };
        self.messages.push(message.clone());

        if let Some(sender) = &mut self.ws_sender {
            if let Some(msg) = KimchiMessage::to_str(&message) {
                sender.send(WsMessage::Text(msg));
            }
        }
        self.msg_to_send.clear();
    }

    fn send_video(&mut self, video: &Vec<u8>) {
        let message = KimchiMessage::Video {
            user: self.current_user.clone(),
            data: video.to_owned(),//video.clone(),
        };

        if let Some(sender) = &mut self.ws_sender {
            if let Some(msg) = KimchiMessage::to_str(&message) {
                sender.send(WsMessage::Text(msg));
            }
        }
    }

    fn set_user(&mut self) {
        match ewebsock::connect(&self.server_url) {
        // match ewebsock::connect_with_wakeup(server_url, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                println!("ws connected");
                self.ws_sender = Some(ws_sender);
                self.ws_receiver = Some(ws_receiver);
            }
            Err(_error) => {
                println!("ws error ");
            }
        }

        let user = &self.user_to_set.clone();
        self.current_user = user.to_owned();
        self.user_to_set.clear();

        let joined_message = KimchiMessage::Joined {
            user: user.to_owned(),
        };
        if let Some(sender) = &mut self.ws_sender {
            if let Some(msg) = KimchiMessage::to_str(&joined_message) {
                sender.send(WsMessage::Text(msg));
            }
        }
    }
}

impl eframe::App for KimchiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.current_user.is_empty() {
                ui.heading(String::from("Set your nick name"));
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.user_to_set);
                    if ui.button("Set ").clicked() {
                        self.set_user();
                    }
                });
            } else {
                chat::Chat::ui(self, ui);
            }

            if let Some(camera) = &mut self.camera {
                if let Ok(buffer) = camera.frame() {
                    let decoded = buffer.decode_image::<RgbFormat>().unwrap();
                    let raw = decoded.as_raw();
                    let color_image = ColorImage::from_rgb([320,240],&decoded);

                    let tex = ui
                        .ctx()
                        .load_texture("frame", color_image,Default::default());

                    ui.image(&tex,tex.size_vec2());
                    // broadcast webcam 
                    self.send_video(raw);
                }
            } 
        });
        ctx.request_repaint();
    }
}
