// use cursorlib::models::calls::audio_call::AudioCall;
// use tokio::sync::mpsc::Receiver;

// pub async fn handle_call(rx: Receiver<(AudioCall, u32)>) {
//     tokio::select! {
//         Some((call, data_len)) = rx.recv() => {
//             println!("Len: {data_len:#?}\nCall: {call:#?}");
//         }
//         result = Future::true => {}
//     }
// }
