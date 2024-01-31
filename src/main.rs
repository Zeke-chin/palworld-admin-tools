use arboard::Clipboard;
use device_query::{DeviceQuery, DeviceState, Keycode};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::HashMap;
use std::sync::mpsc;

use std::{
    thread,
    thread::sleep,
    time::Duration,
    io::Cursor,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum CMD {
    // ShowPlayers
    ShowPlayers,
    // TeleportToPlayer
    MeToPlayer,
    // TeleportToMe
    PlayerToMe,
}

struct AudioPlayer {
    cmd_sender: mpsc::Sender<(CMD, mpsc::Sender<()>)>,
}
impl AudioPlayer {
    fn new() -> Self {
        let (cmd_sender, cmd_receiver) = mpsc::channel::<(CMD, mpsc::Sender<()>)>();
        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let mut sounds = HashMap::new();
            
            // Load sounds
            sounds.insert(CMD::ShowPlayers, include_bytes!("../audios/ShowPlayers.mp3").to_vec());
            sounds.insert(CMD::MeToPlayer, include_bytes!("../audios/TeleportToPlayer.mp3").to_vec());
            sounds.insert(CMD::PlayerToMe, include_bytes!("../audios/TeleportToMe.mp3").to_vec());

            for (cmd, done_sender) in cmd_receiver {
                if let Some(sound_data) = sounds.get(&cmd) {
                    let cursor = Cursor::new(sound_data.clone());
                    let source = Decoder::new(cursor).unwrap();
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source);
                    sink.sleep_until_end();
                    done_sender.send(()).unwrap();
                }
            }
        });

        AudioPlayer { cmd_sender }
    }

    fn play(&self, cmd: CMD) {
        let (done_sender, done_receiver) = mpsc::channel();
        self.cmd_sender.send((cmd, done_sender)).unwrap();
        done_receiver.recv().unwrap(); // Wait for the sound to finish playing
    }
}


fn match_cmd(keys: Vec<Keycode>) -> Option<CMD> {
    if keys.len() != 2 {
        return None;
    }
    println!("keys:\t{:?}", keys);
    if keys[1] == Keycode::BackSlash {
        match keys[0] {
            Keycode::S => Some(CMD::ShowPlayers),
            Keycode::P => Some(CMD::MeToPlayer),
            Keycode::M => Some(CMD::PlayerToMe),
            _ => None
        }
    } else {
        None
    }
}

fn to_palyer(text: String) -> i32 {
    // 判断是否为一个 长度为9 的数字 如：128991414
    if text.len() == 9 {
        if text.parse::<i32>().is_ok() {
            return text.parse::<i32>().unwrap();
        }
    }
    // print!("剪切板内容并不是一个player id\t{}", text);
    return 0;
}


fn main() {
    let mut clipboard: Clipboard = Clipboard::new().unwrap();
    let device_state = DeviceState::new();
    let mut cmd_str = "".to_string();
    let audio_player = AudioPlayer::new();


    loop {
        // let mut clipboard: Clipboard = Clipboard::new().unwrap();
        sleep(Duration::from_millis(100));
        let keys = device_state.get_keys();
        let cmd_type = match_cmd(keys);
        let clip_txt = clipboard.get_text().unwrap_or("".to_string());
        let player_id = to_palyer(clip_txt);

        // 输出 cmd_type
        println!("cmd_type:\t{:?}", cmd_type);
        match cmd_type {
            Some(CMD::ShowPlayers) => {
                cmd_str = "ShowPlayers".to_string();
                audio_player.play(CMD::ShowPlayers);
            }
            Some(CMD::MeToPlayer) => {
                if player_id == 0 {
                    continue;
                }
                cmd_str = format!("TeleportToPlayer {}", player_id);
                audio_player.play(CMD::MeToPlayer);
            }
            Some(CMD::PlayerToMe) => {
                if player_id == 0 {
                    continue;
                }
                cmd_str = format!("TeleportToMe {}", player_id);
                audio_player.play(CMD::PlayerToMe);
            }
            _ => {
                cmd_str = "".to_string();
            }
        }
        if cmd_str == "".to_string() {
            continue;
        }
        println!("CMD:\t{}", cmd_str);
        clipboard.set_text(cmd_str).unwrap();
    }
}

//
// use rodio::{Decoder, OutputStream, source::Source};
// use std::io::Cursor;
//
// fn main() {
//     // 获得输出音频流和声道句柄
//     let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//     // 这里我们将用到的音乐数据直接嵌入到代码中，作为一个字节数组
//     // 这是通过将文件内容转化为字节数组得到的
//     // 例如：你可以使用 `include_bytes!` 宏来实现
//     let music_data = include_bytes!("/Users/zeke/Downloads/2024-01-31-142741_176972.mp3");
//
//     // 使用 `Cursor` 来包装音乐数据的字节数组
//     let cursor = Cursor::new(music_data.as_ref());
//     // 使用 `rodio` 解码音乐流
//     let source = Decoder::new(cursor).unwrap();
//     // 播放音乐
//     println!("开始播放音乐");
//     stream_handle.play_raw(source.convert_samples()).unwrap();
//     println!("播放音乐结束");
//
//     // 音频是非阻塞的，所以我们需要这样阻塞主线程以继续播放
//     std::thread::sleep(std::time::Duration::from_secs(10));
// }